use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;

use kissat::statik::Kissat;
use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};

pub struct KissatStaticSolver {
    inner: Kissat,
    nvars: usize,
    nclauses: usize,
}

impl KissatStaticSolver {
    pub fn new(inner: Kissat) -> Self {
        Self {
            inner,
            nvars: 0,
            nclauses: 0,
        }
    }
}

impl Default for KissatStaticSolver {
    fn default() -> Self {
        Self::new(Kissat::new())
    }
}

impl From<Kissat> for KissatStaticSolver {
    fn from(inner: Kissat) -> Self {
        KissatStaticSolver::new(inner)
    }
}

impl Debug for KissatStaticSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KissatStaticSolver").field("inner", &self.inner).finish()
    }
}

impl Display for KissatStaticSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", tynm::type_name::<Self>(), self.inner)
    }
}

impl Solver for KissatStaticSolver {
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

    fn assume<L>(&mut self, _lit: L)
    where
        L: Into<Lit>,
    {
        panic!("Kissat does not support assumptions");
    }

    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        self.nclauses += 1;
        self.inner.add_clause(lits.into_iter().map_into::<Lit>());
    }

    fn solve(&mut self) -> SolveResponse {
        use kissat::SolveResponse as KissatSolveResponse;
        match self.inner.solve() {
            KissatSolveResponse::Sat => SolveResponse::Sat,
            KissatSolveResponse::Unsat => SolveResponse::Unsat,
            KissatSolveResponse::Interrupted => SolveResponse::Unknown,
        }
    }

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        use kissat::LitValue as KissatLitValue;
        match self.inner.value(lit.into().into()) {
            KissatLitValue::True => LitValue::True,
            KissatLitValue::False => LitValue::False,
            _ => panic!("Unexpected value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_kissat_static() -> color_eyre::Result<()> {
        let mut solver = KissatStaticSolver::default();
        assert!(solver.signature().contains("kissat"));

        // Initializing variables
        let a = solver.new_var();
        let b = solver.new_var();
        let c = solver.new_var();
        let d = solver.new_var();
        assert_eq!(solver.num_vars(), 4);
        assert_eq!(a.get(), 1);
        assert_eq!(b.get(), 2);
        assert_eq!(c.get(), 3);
        assert_eq!(d.get(), 4);

        // Adding [(a or b) and (c or d) and not(a and b) and not(c and d)]
        solver.add_clause([a, b]);
        solver.add_clause(&[c, d]);
        solver.add_clause(vec![-a, -b]);
        solver.add_clause(&vec![-c, -d]);

        // Problem is satisfiable
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        // Note: Kissat currently does not support assumptions.
        //
        // // Assuming both a and b to be true
        // solver.assume(a);
        // solver.assume(b);
        // // Problem is unsatisfiable under assumptions
        // let response = solver.solve();
        // assert_eq!(response, SolveResponse::Unsat);
        //
        // // `solve` resets assumptions, so calling it again should produce SAT
        // let response = solver.solve();
        // assert_eq!(response, SolveResponse::Sat);

        Ok(())
    }
}
