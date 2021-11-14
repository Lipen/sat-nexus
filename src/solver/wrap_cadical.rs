use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;

use crate::cadical::less::CadicalSolver2;
use crate::context::Context;
use crate::core::lit::Lit;
use crate::ipasir::{LitValue, SolveResponse, SolverError};
use crate::solver::Solver;

pub struct WrappedCadicalSolver {
    inner: CadicalSolver2,
    context: Rc<RefCell<Context>>,
    nvars: usize,
    nclauses: usize,
}

impl WrappedCadicalSolver {
    pub fn new() -> Self {
        Self::new_custom(CadicalSolver2::new())
    }

    pub fn new_custom(inner: CadicalSolver2) -> Self {
        Self {
            inner,
            context: Rc::new(RefCell::new(Context::new())),
            nvars: 0,
            nclauses: 0,
        }
    }
}

impl Default for WrappedCadicalSolver {
    fn default() -> Self {
        WrappedCadicalSolver::new()
    }
}

impl From<CadicalSolver2> for WrappedCadicalSolver {
    fn from(inner: CadicalSolver2) -> Self {
        WrappedCadicalSolver::new_custom(inner)
    }
}

impl Solver for WrappedCadicalSolver {
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
        self.inner.add_clause(lits.into_iter().map_into::<Lit>());
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.inner.assume(lit.into().into());
    }

    fn solve(&mut self) -> SolveResponse {
        match self.inner.solve() {
            0 => Ok(SolveResponse::Interrupted),
            10 => Ok(SolveResponse::Sat),
            20 => Ok(SolveResponse::Unsat),
            invalid => Err(SolverError::InvalidResponseSolve { value: invalid }),
        }
        .unwrap_or_else(|e| panic!("Could not solve: {}", e))
    }

    fn val<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        let lit = lit.into();
        match self.inner.val(lit.into()) {
            0 => Ok(LitValue::DontCare),
            p if p == lit.get() => Ok(LitValue::True),
            n if n == -lit.get() => Ok(LitValue::False),
            invalid => Err(SolverError::InvalidResponseVal {
                lit: lit.into(),
                value: invalid,
            }),
        }
        .unwrap_or_else(|e| panic!("Could not get literal value: {}", e))
    }
}

impl WrappedCadicalSolver {
    pub fn add_unit<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.add_clause([lit]);
    }
}

impl fmt::Display for WrappedCadicalSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WrappedSolver({})", self.signature())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_solver() -> color_eyre::Result<()> {
        let mut solver = WrappedCadicalSolver::new();
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
