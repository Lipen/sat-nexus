use itertools::Itertools;

use crate::lit::Lit;
use crate::solver::Solver;

use super::Ops;

impl<S> Encodings for S where S: Solver + ?Sized {}

pub trait Encodings: Solver {
    fn encode_onehot(&mut self, lits: &[Lit]) {
        self.encode_at_least_one(lits);
        self.encode_at_most_one(lits);
    }

    fn encode_at_least_one(&mut self, lits: &[Lit]) {
        self.add_clause(lits.iter().copied());
    }

    fn encode_at_most_one(&mut self, lits: &[Lit]) {
        for (&a, &b) in lits.iter().tuple_combinations() {
            self.imply(a, -b);
        }
    }
}
