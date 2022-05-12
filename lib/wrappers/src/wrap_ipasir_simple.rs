use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use easy_ext::ext;

use ipasir::Ipasir;
use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SimpleSolver, SolveResponse};

pub struct IpasirSimpleSolver {
    inner: Ipasir,
    nvars: usize,
    nclauses: usize,
}

impl IpasirSimpleSolver {
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

impl From<Ipasir> for IpasirSimpleSolver {
    fn from(inner: Ipasir) -> Self {
        IpasirSimpleSolver::new(inner)
    }
}

impl Display for IpasirSimpleSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IpasirSolver({})", self.signature())
    }
}

impl SimpleSolver for IpasirSimpleSolver {
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

    fn assume(&mut self, lit: Lit) {
        self.inner.assume(lit.to_ipasir());
    }

    fn add_clause(&mut self, lits: &[Lit]) {
        self.nclauses += 1;
        self.inner.add_clause(lits.iter().map(|x| x.to_ipasir()));
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

    fn value(&self, lit: Lit) -> LitValue {
        match self.inner.val(lit.to_ipasir()) {
            Ok(ipasir::LitValue::True) => LitValue::True,
            Ok(ipasir::LitValue::False) => LitValue::False,
            Ok(ipasir::LitValue::DontCare) => LitValue::DontCare,
            Err(e) => panic!("Could not get literal value: {}", e),
        }
    }
}

#[ext]
impl Lit {
    fn to_ipasir(self) -> ipasir::Lit {
        unsafe { ipasir::Lit::new_unchecked(self.into()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_ipasir() -> color_eyre::Result<()> {
        let mut solver = IpasirSimpleSolver::new_cadical();
        assert!(solver.signature().contains("cadical"));

        // Initializing variables
        let a = solver.new_var();
        let b = solver.new_var();
        let c = solver.new_var();
        let d = solver.new_var();
        assert_eq!(4, solver.num_vars());

        // Adding [(a or b) and (c or d) and not(a and b) and not(c and d)]
        solver.add_clause(&[a, b]);
        solver.add_clause(&[c, d]);
        solver.add_clause(&[-a, -b]);
        solver.add_clause(&[-c, -d]);

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
