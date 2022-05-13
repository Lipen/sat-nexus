use itertools::Itertools;

use crate::lit::Lit;
use crate::op::ops::Ops;
use crate::solver::Solver;

impl<S> Encodings for S where S: Solver {}

pub trait Encodings: Solver + Sized {
    fn encode_onehot(&mut self, lits: &[Lit]) {
        self.encode_at_least_one(lits);
        self.encode_at_most_one(lits);
    }

    fn encode_at_least_one(&mut self, lits: &[Lit]) {
        self.add_clause(lits);
    }

    fn encode_at_most_one(&mut self, lits: &[Lit]) {
        for (&a, &b) in lits.iter().tuple_combinations() {
            self.imply(a, -b);
        }
    }
}
