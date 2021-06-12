use totalizer::Totalizer;

use crate::solver::GenericSolver;
use crate::types::Lit;

pub mod totalizer;

trait Cardinality: GenericSolver {
    fn declare_totalizer(&mut self, input_vars: &[Lit]) -> Totalizer
    where
        Self: Sized,
    {
        Totalizer::declare(self, input_vars)
    }
}

impl<S: GenericSolver> Cardinality for S {}
