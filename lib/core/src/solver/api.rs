use std::borrow::Cow;

use crate::lit::Lit;

use super::types::*;

// Note: `Solver` trait is NOT object-safe.
// const _: Option<&dyn Solver> = None; // doesn't compile, see `rustc --explain E0038`

pub trait Solver: Sized {
    /// Return the signature of the solver as a `Cow<str>`.
    fn signature(&self) -> Cow<str>;

    /// Reset the solver to its initial state.
    fn reset(&mut self);

    /// Release any resources held by the solver.
    fn release(&mut self);

    /// Return the number of variables in the solver.
    fn num_vars(&self) -> usize;

    /// Return the number of clauses in the solver.
    fn num_clauses(&self) -> usize;

    /// Create a new variable in the solver and return its literal representation.
    fn new_var(&mut self) -> Lit;

    /// Add an assumption to the solver.
    /// The assumption is represented by the given literal.
    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>;

    /// Add a clause to the solver.
    /// The clause is represented by an iterator of literals.
    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>;

    /// Add a clause to the solver.
    /// The clause is represented by a slice of literals.
    fn add_clause_<L>(&mut self, lits: &[L])
    where
        L: Into<Lit> + Copy,
    {
        self.add_clause(lits)
    }

    /// Add a unit clause to the solver.
    fn add_unit<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.add_clause_(&[lit.into()])
    }

    /// Solve the problem given to the solver and return the result as a [SolveResponse].
    fn solve(&mut self) -> SolveResponse;

    /// Return the value of the given literal in the solver.
    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>;
}

impl<S> crate::op::ops::AddClause for S
where
    S: Solver,
{
    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        self.add_clause(lits)
    }
}
