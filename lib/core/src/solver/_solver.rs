use std::borrow::Cow;
use std::fmt::Display;

use itertools::Itertools;

use crate::lit::Lit;

use super::_types::*;

// `BaseSolver` trait is object-safe.
const _: Option<&dyn BaseSolver> = None;

pub trait BaseSolver {
    fn assume_(&mut self, lit: Lit);
    fn value_(&self, lit: Lit) -> LitValue;
    fn add_clause_(&mut self, lits: &[Lit]);
    fn add_clause__(&mut self, lits: &mut dyn Iterator<Item = Lit>);
}

impl<S> BaseSolver for Box<S>
where
    S: BaseSolver + ?Sized,
{
    fn assume_(&mut self, lit: Lit) {
        (**self).assume_(lit)
    }

    fn value_(&self, lit: Lit) -> LitValue {
        (**self).value_(lit)
    }

    fn add_clause_(&mut self, lits: &[Lit]) {
        (**self).add_clause_(lits)
    }

    fn add_clause__(&mut self, lits: &mut dyn Iterator<Item = Lit>) {
        (**self).add_clause__(lits)
    }
}

//===================================================================

// `Solver` trait is object-safe.
const _: Option<&dyn Solver> = None;

pub trait Solver: BaseSolver + Display {
    fn signature(&self) -> Cow<str>;

    fn reset(&mut self);
    fn release(&mut self);

    fn num_vars(&self) -> usize;
    fn num_clauses(&self) -> usize;

    fn new_var(&mut self) -> Lit;

    fn assume<L>(&mut self, lit: L)
    where
        Self: Sized,
        L: Into<Lit>,
    {
        self.assume_(lit.into());
    }

    // Note: it is strongly recommended to implement this method and not rely on the default impl.
    fn add_clause<I>(&mut self, lits: I)
    where
        Self: Sized,
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        self.add_clause__(&mut lits.into_iter().map_into::<Lit>());
    }

    fn add_unit<L>(&mut self, lit: L)
    where
        Self: Sized,
        L: Into<Lit>,
    {
        self.add_clause_(&[lit.into()]);
    }

    fn solve(&mut self) -> SolveResponse;

    fn value<L>(&self, lit: L) -> LitValue
    where
        Self: Sized,
        L: Into<Lit>,
    {
        self.value_(lit.into())
    }
}

impl<S> Solver for Box<S>
where
    S: Solver + ?Sized,
{
    fn signature(&self) -> Cow<str> {
        // equivalent: Solver::signature(&**self)
        (**self).signature()
    }

    fn reset(&mut self) {
        (**self).reset()
    }

    fn release(&mut self) {
        (**self).release()
    }

    fn num_vars(&self) -> usize {
        (**self).num_vars()
    }

    fn num_clauses(&self) -> usize {
        (**self).num_clauses()
    }

    fn new_var(&mut self) -> Lit {
        (**self).new_var()
    }

    fn solve(&mut self) -> SolveResponse {
        (**self).solve()
    }
}
