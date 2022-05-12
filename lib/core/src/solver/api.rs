use std::borrow::Cow;
use std::fmt::Display;

use crate::lit::Lit;

use super::types::*;

// Note: `Solver` trait is NOT object-safe, by design.
// const _: Option<&dyn Solver> = None; // compiles when trait is object-safe

pub trait Solver: Sized + Display {
    fn signature(&self) -> Cow<str>;

    fn reset(&mut self);
    fn release(&mut self);

    fn num_vars(&self) -> usize;
    fn num_clauses(&self) -> usize;

    fn new_var(&mut self) -> Lit;

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>;

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

    fn solve(&mut self) -> SolveResponse;

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>;

    // TODO: model
}
