use std::borrow::Cow;
use std::fmt::Display;

use crate::lit::Lit;

use super::types::*;

/// Generic SAT-solver interface.
///
/// ---
///
/// **Note:** `Solver` trait is NOT object-safe, by design.
///
/// ```compile_fail
/// # use sat_nexus_core::solver::Solver;
/// // should not compile since `Solver` is not object-safe
/// let _: Option<&dyn Solver> = None;
/// ```
///
/// If you need a generic `Solver` implementation,
/// use `DelegatingSolver` which delegates to `DispatchingSolver`,
/// which, in turn, implements object-safe `SimpleSolver` trait.
///
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
