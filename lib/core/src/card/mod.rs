use totalizer::Totalizer;

use crate::lit::Lit;
use crate::solver::Solver;

pub mod totalizer;

impl<S> Cardinality for S where S: Solver {}

pub trait Cardinality: Solver {
    fn declare_totalizer(&mut self, input_vars: &[Lit]) -> Totalizer
    where
        Self: Sized,
    {
        Totalizer::declare(self, input_vars)
    }
}
