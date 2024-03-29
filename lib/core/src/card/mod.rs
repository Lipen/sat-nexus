use totalizer::Totalizer;

use crate::lit::Lit;
use crate::solver::Solver;

pub mod totalizer;

impl<S> Cardinality for S where S: Solver {}

pub trait Cardinality: Solver + Sized {
    fn declare_totalizer(&mut self, input_vars: &[Lit]) -> Totalizer {
        Totalizer::new(self, input_vars)
    }
}
