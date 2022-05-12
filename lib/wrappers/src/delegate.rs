use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SimpleSolver, SolveResponse, Solver};

use crate::dispatch::DispatchingSolver;

pub struct DelegatingSolver {
    inner: DispatchingSolver,
}

impl DelegatingSolver {
    pub fn new(inner: DispatchingSolver) -> Self {
        Self { inner }
    }

    pub fn new_minisat() -> Self {
        Self::new(DispatchingSolver::new_minisat())
    }
    pub fn new_cadical() -> Self {
        Self::new(DispatchingSolver::new_cadical())
    }

    pub fn by_name(name: &str) -> Self {
        Self::new(DispatchingSolver::by_name(name))
    }
}

impl Display for DelegatingSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DelegatingSolver({})", self.inner)
    }
}

impl Solver for DelegatingSolver {
    fn signature(&self) -> Cow<str> {
        self.inner.signature()
    }

    fn reset(&mut self) {
        self.inner.reset()
    }

    fn release(&mut self) {
        self.inner.release()
    }

    fn num_vars(&self) -> usize {
        self.inner.num_vars()
    }

    fn num_clauses(&self) -> usize {
        self.inner.num_clauses()
    }

    fn new_var(&mut self) -> Lit {
        self.inner.new_var()
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.inner.assume(lit.into())
    }

    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        self.inner.add_clause(&lits.into_iter().map_into().collect_vec())
    }

    fn solve(&mut self) -> SolveResponse {
        self.inner.solve()
    }

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        self.inner.value(lit.into())
    }
}
