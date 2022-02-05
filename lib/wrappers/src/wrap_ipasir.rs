use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;

use ipasir::solver::IpasirSolver;
use ipasir::Ipasir;
use sat_nexus_core::context::Context;
use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};

trait Ext {
    fn to_ipasir(self) -> ipasir::Lit;
}

impl Ext for Lit {
    fn to_ipasir(self) -> ipasir::Lit {
        unsafe { ipasir::Lit::new_unchecked(self.into()) }
    }
}

pub struct WrappedIpasirSolver<S>
where
    S: Ipasir,
{
    inner: S,
    context: Rc<RefCell<Context>>,
    nvars: usize,
    nclauses: usize,
}

impl WrappedIpasirSolver<IpasirSolver> {
    pub fn new(inner: IpasirSolver) -> Self {
        Self {
            inner,
            context: Rc::new(RefCell::new(Context::new())),
            nvars: 0,
            nclauses: 0,
        }
    }

    pub fn new_cadical() -> Self {
        Self::new(IpasirSolver::new_cadical())
    }
    pub fn new_minisat() -> Self {
        Self::new(IpasirSolver::new_minisat())
    }
    pub fn new_glucose() -> Self {
        Self::new(IpasirSolver::new_glucose())
    }
}

impl From<IpasirSolver> for WrappedIpasirSolver<IpasirSolver> {
    fn from(inner: IpasirSolver) -> Self {
        WrappedIpasirSolver::new(inner)
    }
}

impl fmt::Display for WrappedIpasirSolver<IpasirSolver> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WrappedSolver({})", self.signature())
    }
}

impl Solver for WrappedIpasirSolver<IpasirSolver> {
    fn signature(&self) -> Cow<str> {
        self.inner.signature().into()
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
    fn release(&mut self) {
        self.inner.release();
    }

    fn context(&self) -> Rc<RefCell<Context>> {
        Rc::clone(&self.context)
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

    fn add_clause<I, L>(&mut self, lits: I)
    where
        I: IntoIterator<Item = L>,
        L: Into<Lit>,
    {
        self.nclauses += 1;
        self.inner
            .add_clause(lits.into_iter().map_into::<Lit>().map(|x| x.to_ipasir()));
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.inner.assume(lit.into().to_ipasir());
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

    fn val<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        match self.inner.val(lit.into().to_ipasir()) {
            Ok(ipasir::LitValue::True) => LitValue::True,
            Ok(ipasir::LitValue::False) => LitValue::False,
            Ok(ipasir::LitValue::DontCare) => LitValue::DontCare,
            Err(e) => panic!("Could not get literal value: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_solver() -> color_eyre::Result<()> {
        let mut solver = WrappedIpasirSolver::new_cadical();
        assert!(solver.signature().contains("cadical"));

        // Adding [(1 or 2) and (3 or 4) and not(1 and 2) and not(3 and 4)]
        solver.add_clause([1, 2]);
        solver.add_clause(&[3, 4]);
        solver.add_clause(vec![-1, -2]);
        solver.add_clause(&vec![-3, -4]);

        // Problem is satisfiable
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        // Assuming both 1 and 2 to be true
        solver.assume(1);
        solver.assume(&2);
        // Problem is unsatisfiable under assumptions
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Unsat);

        // `solve` resets assumptions, so calling it again should produce SAT
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        Ok(())
    }
}
