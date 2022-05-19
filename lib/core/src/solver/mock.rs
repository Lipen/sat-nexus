use std::borrow::Cow;
use std::fmt::{Display, Formatter};

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

impl Display for MockSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", tynm::type_name::<Self>(), self.signature())
    }
}

impl Solver for MockSolver {
    fn signature(&self) -> Cow<str> {
        "mock".into()
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
        let lits = lits.into_iter().map_into::<Lit>().collect_vec();
        self.clauses.push(lits);
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
    fn test_mock_solver() -> color_eyre::Result<()> {
        let mut solver = MockSolver::new();
        assert_eq!(solver.signature(), "mock");

        let a = solver.new_var();
        let b = solver.new_var();
        let c = solver.new_var();
        let d = solver.new_var();
        assert_eq!(solver.nvars, 4);

        solver.add_clause([a, b]);
        solver.add_clause(&[c, d]);
        solver.add_clause(vec![-a, -b]);
        solver.add_clause(&vec![-c, -d]);
        solver.add_unit(a);
        solver.add_unit(-c);
        assert_eq!(solver.nclauses, 6);

        Ok(())
    }
}
