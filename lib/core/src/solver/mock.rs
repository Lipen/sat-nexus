use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

use crate::lit::Lit;
use crate::solver::BaseSolver;

use super::Solver;
use super::_types::*;

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

impl BaseSolver for MockSolver {
    fn assume_(&mut self, _lit: Lit) {
        // TODO
    }

    fn value_(&self, _lit: Lit) -> LitValue {
        // TODO
        LitValue::False
    }

    fn add_clause_(&mut self, lits: &[Lit]) {
        self.nclauses += 1;
        self.clauses.push(lits.to_vec());
    }

    fn add_clause__(&mut self, lits: &mut dyn Iterator<Item = Lit>) {
        self.nclauses += 1;
        self.clauses.push(lits.collect_vec());
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

    fn add_clause<I>(&mut self, lits: I)
    where
        Self: Sized,
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        self.nclauses += 1;
        self.clauses.push(lits.into_iter().map_into::<Lit>().collect_vec());
    }

    fn solve(&mut self) -> SolveResponse {
        // TODO
        SolveResponse::Sat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_solver() -> color_eyre::Result<()> {
        let mut solver = MockSolver::new();
        assert_eq!(solver.signature(), "mock");

        let a = solver.new_var();
        let b = solver.new_var();
        let c = solver.new_var();
        let d = solver.new_var();
        assert_eq!(solver.nvars, 4);

        solver.add_clause_(&[a, b]);
        solver.add_clause_(&[c, d]);
        solver.add_clause_(&[-a, -b]);
        solver.add_clause_(&[-c, -d]);
        solver.add_clause_(&[a]);
        solver.add_clause_(&[-c]);
        assert_eq!(solver.nclauses, 6);

        Ok(())
    }
}
