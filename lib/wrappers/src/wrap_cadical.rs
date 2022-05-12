use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

use cadical::Cadical;
use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};

pub struct WrappedCadical {
    inner: Cadical,
    nvars: usize,
    nclauses: usize,
}

impl WrappedCadical {
    pub fn new() -> Self {
        Self::new_custom(Cadical::new())
    }

    pub fn new_custom(inner: Cadical) -> Self {
        Self {
            inner,
            nvars: 0,
            nclauses: 0,
        }
    }
}

impl Default for WrappedCadical {
    fn default() -> Self {
        WrappedCadical::new()
    }
}

impl From<Cadical> for WrappedCadical {
    fn from(inner: Cadical) -> Self {
        WrappedCadical::new_custom(inner)
    }
}

impl Display for WrappedCadical {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "WrappedSolver({})", self.inner)
    }
}

impl Solver for WrappedCadical {
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
        self.inner.assume(lit.into().into());
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
        use cadical::SolveResponse as CadicalSolveResponse;
        match self.inner.solve() {
            Ok(CadicalSolveResponse::Sat) => SolveResponse::Sat,
            Ok(CadicalSolveResponse::Unsat) => SolveResponse::Unsat,
            Ok(CadicalSolveResponse::Interrupted) => SolveResponse::Unknown,
            Err(e) => panic!("Could not solve: {}", e),
        }
    }

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        use cadical::LitValue as CadicalLitValue;
        match self.inner.val(lit.into().into()) {
            Ok(CadicalLitValue::True) => LitValue::True,
            Ok(CadicalLitValue::False) => LitValue::False,
            Err(e) => panic!("Could not get literal value: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_cadical() -> color_eyre::Result<()> {
        let mut solver = WrappedCadical::new();
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
