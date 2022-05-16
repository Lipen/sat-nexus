use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use itertools::Itertools;
use tap::Pipe;

use ipasir::Ipasir;
use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};

pub struct IpasirSolver {
    inner: Ipasir,
    nvars: usize,
    nclauses: usize,
}

impl IpasirSolver {
    pub fn new(inner: Ipasir) -> Self {
        Self {
            inner,
            nvars: 0,
            nclauses: 0,
        }
    }

    pub fn new_cadical() -> Self {
        Self::new(Ipasir::new_cadical())
    }
    pub fn new_minisat() -> Self {
        Self::new(Ipasir::new_minisat())
    }
    pub fn new_glucose() -> Self {
        Self::new(Ipasir::new_glucose())
    }
}

impl From<Ipasir> for IpasirSolver {
    fn from(inner: Ipasir) -> Self {
        IpasirSolver::new(inner)
    }
}

impl Display for IpasirSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", tynm::type_name::<Self>(), self.inner)
    }
}

impl Solver for IpasirSolver {
    fn signature(&self) -> Cow<str> {
        self.inner.signature().into()
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
    fn release(&mut self) {
        self.inner.release();
    }

    fn num_vars(&self) -> usize {
        self.nvars
    }
    fn num_clauses(&self) -> usize {
        self.nclauses
    }

    fn new_var(&mut self) -> Lit {
        self.nvars += 1;
        Lit::new(self.nvars as i32)
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.inner.assume(lit.into().pipe(to_ipasir));
    }

    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        self.nclauses += 1;
        self.inner.add_clause(lits.into_iter().map_into::<Lit>().map(to_ipasir));
    }

    fn solve(&mut self) -> SolveResponse {
        match self.inner.solve() {
            Ok(ipasir::SolveResponse::Sat) => SolveResponse::Sat,
            Ok(ipasir::SolveResponse::Unsat) => SolveResponse::Unsat,
            Ok(ipasir::SolveResponse::Interrupted) => SolveResponse::Unknown,
            Err(e) => {
                eprintln!("Could not solve: {}", e);
                SolveResponse::Unknown
            }
        }
    }

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        match self.inner.val(lit.into().pipe(to_ipasir)) {
            Ok(ipasir::LitValue::True) => LitValue::True,
            Ok(ipasir::LitValue::False) => LitValue::False,
            Ok(ipasir::LitValue::DontCare) => LitValue::DontCare,
            Err(e) => panic!("Could not get literal value: {}", e),
        }
    }
}

fn to_ipasir(lit: Lit) -> ipasir::Lit {
    unsafe { ipasir::Lit::new_unchecked(lit.into()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_ipasir() -> color_eyre::Result<()> {
        let mut solver = IpasirSolver::new_cadical();
        assert!(solver.signature().contains("cadical"));

        // Initializing variables
        let a = solver.new_var();
        let b = solver.new_var();
        let c = solver.new_var();
        let d = solver.new_var();
        assert_eq!(solver.num_vars(), 4);

        // Adding [(a or b) and (c or d) and not(a and b) and not(c and d)]
        solver.add_clause([a, b]);
        solver.add_clause(&[c, d]);
        solver.add_clause(vec![-a, -b]);
        solver.add_clause(&vec![-c, -d]);

        // Problem is satisfiable
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        // Assuming both a and b to be true
        solver.assume(a);
        solver.assume(b);
        // Problem is unsatisfiable under assumptions
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Unsat);

        // `solve` resets assumptions, so calling it again should produce SAT
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        Ok(())
    }
}
