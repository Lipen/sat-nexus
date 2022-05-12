use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

use crate::lit::Lit;
use crate::solver::{LitValue, SimpleSolver, SolveResponse};

pub struct DelegateSolver {
    inner: Box<dyn SimpleSolver>,
}

impl DelegateSolver {
    pub fn new(inner: impl SimpleSolver + 'static) -> Self {
        Self { inner: Box::new(inner) }
    }
}

impl<S> From<S> for DelegateSolver
where
    S: SimpleSolver + 'static,
{
    fn from(s: S) -> Self {
        DelegateSolver::new(s)
    }
}

impl Display for DelegateSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DelegateSolver({})", self.inner)
    }
}

impl DelegateSolver {
    pub fn signature(&self) -> Cow<str> {
        self.inner.signature()
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }

    pub fn release(&mut self) {
        self.inner.release();
    }

    pub fn num_vars(&self) -> usize {
        self.inner.num_vars()
    }

    pub fn num_clauses(&self) -> usize {
        self.inner.num_clauses()
    }

    pub fn new_var(&mut self) -> Lit {
        self.inner.new_var()
    }

    pub fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.inner.assume(lit.into());
    }

    pub fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        let lits = lits.into_iter().map_into().collect_vec();
        self.inner.add_clause(&lits);
    }

    pub fn solve(&mut self) -> SolveResponse {
        self.inner.solve()
    }

    pub fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        self.inner.value(lit.into())
    }
}
