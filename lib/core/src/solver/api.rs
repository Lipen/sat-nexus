use std::borrow::Cow;

use crate::lit::Lit;

use super::types::*;

pub trait Solver {
    fn signature(&self) -> Cow<str>;

    fn reset(&mut self);
    fn release(&mut self);

    fn num_vars(&self) -> usize;
    fn num_clauses(&self) -> usize;

    fn new_var(&mut self) -> Lit;

    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>;

    fn add_unit<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.add_clause([lit]);
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>;

    fn solve(&mut self) -> SolveResponse;

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>;

    // TODO: model
}
