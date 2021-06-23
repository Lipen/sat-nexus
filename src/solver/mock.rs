use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;

use crate::context::Context;
use crate::ipasir::{LitValue, SolveResponse};
use crate::solver::GenericSolver;
use crate::types::Lit;

#[derive(Debug)]
pub struct MockSolver {
    context: Rc<RefCell<Context>>,
    nvars: usize,
    nclauses: usize,
    clauses: Vec<Vec<Lit>>,
    tmp_clause: Vec<Lit>,
}

impl MockSolver {
    pub fn new() -> Self {
        Self {
            context: Rc::new(RefCell::new(Context::new())),
            nvars: 0,
            nclauses: 0,
            clauses: Vec::new(),
            tmp_clause: Vec::new(),
        }
    }
}

impl fmt::Display for MockSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.signature())
    }
}

impl GenericSolver for MockSolver {
    fn signature(&self) -> Cow<str> {
        "MockSolver".into()
    }

    fn reset(&mut self) {
        /* do nothing */
    }
    fn release(&mut self) {
        /* do nothing */
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
        Lit::from(self.nvars)
    }

    fn add_clause<I, L>(&mut self, lits: I)
    where
        I: IntoIterator<Item = L>,
        L: Into<Lit>,
    {
        self.nclauses += 1;
        self.clauses
            .push(lits.into_iter().map_into::<Lit>().collect());
    }

    fn add_clause_lit<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.tmp_clause.push(lit.into());
    }

    fn finalize_clause(&mut self) {
        let lits = std::mem::take(&mut self.tmp_clause);
        self.add_clause(lits);
    }

    fn assume<L>(&mut self, _lit: L)
    where
        L: Into<Lit>,
    {
        todo!()
    }

    fn solve(&mut self) -> SolveResponse {
        // TODO
        SolveResponse::Sat
    }

    fn val<L>(&self, _lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_solver() -> color_eyre::Result<()> {
        let mut solver = MockSolver::new();
        assert_eq!(solver.signature(), "MockSolver");

        for i in 1..=4 {
            let var = solver.new_var();
            assert_eq!(var.get(), i)
        }
        assert_eq!(solver.nvars, 4);

        solver.add_clause([1, 2]);
        solver.add_clause([3, 4]);
        solver.add_clause([-1, -2]);
        solver.add_clause([-3, -4]);
        solver.add_clause([1]);
        solver.add_clause([-3]);
        assert_eq!(solver.nclauses, 6);

        Ok(())
    }
}
