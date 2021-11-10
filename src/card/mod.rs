use totalizer::Totalizer;

use crate::solver::Solver;
use crate::types::Lit;

pub mod totalizer;

trait Cardinality: Solver {
    fn declare_totalizer(&mut self, input_vars: &[Lit]) -> Totalizer
    where
        Self: Sized,
    {
        Totalizer::declare(self, input_vars)
    }
}

impl<S: Solver> Cardinality for S {}
