use std::borrow::Cow;
use std::fmt;

use itertools::Itertools;

use crate::lit::Lit;

use super::types::*;
use super::Solver;

#[derive(Debug)]
pub struct MockSolver {
    nvars: usize,
    nclauses: usize,
    clauses: Vec<Vec<Lit>>,
}

impl MockSolver {
    pub fn new() -> Self {
        Self {
            nvars: 0,
            nclauses: 0,
            clauses: Vec::new(),
        }
    }
}

impl Default for MockSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MockSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.signature())
    }
}

impl Solver for MockSolver {
    fn signature(&self) -> Cow<str> {
        "MockSolver".into()
    }

    fn reset(&mut self) {
        /* do nothing */
    }
    fn release(&mut self) {
        /* do nothing */
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

    fn assume<L>(&mut self, _lit: L)
    where
        L: Into<Lit>,
    {
        // TODO
    }

    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        self.nclauses += 1;
        self.clauses.push(lits.into_iter().map_into::<Lit>().collect());
    }

    fn solve(&mut self) -> SolveResponse {
        // TODO
        SolveResponse::Sat
    }

    fn value<L>(&self, _lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        // TODO
        LitValue::False
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
        solver.add_clause(&[3, 4]);
        solver.add_clause(vec![-1, -2]);
        solver.add_clause(&vec![-3, -4]);
        solver.add_unit(1);
        solver.add_unit(&-3);
        assert_eq!(solver.nclauses, 6);

        Ok(())
    }
}
