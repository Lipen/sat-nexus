use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

use crate::lit::Lit;
use crate::solver::simple::SimpleSolver;
use crate::solver::wrap::WrapSolver;
use crate::solver::{LitValue, SolveResponse, Solver};

pub struct DelegateSolver {
    inner: Box<dyn SimpleSolver>,
}

impl DelegateSolver {
    pub fn new(inner: impl SimpleSolver + 'static) -> Self {
        Self { inner: Box::new(inner) }
    }

    pub fn wrap(inner: impl Solver + 'static) -> Self {
        Self::new(WrapSolver::new(inner))
    }
}

impl From<Box<dyn SimpleSolver>> for DelegateSolver {
    fn from(inner: Box<dyn SimpleSolver>) -> Self {
        DelegateSolver { inner }
    }
}

impl Display for DelegateSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", tynm::type_name::<Self>(), self.signature())
    }
}

impl Solver for DelegateSolver {
    fn signature(&self) -> Cow<str> {
        self.inner.signature()
    }

    fn reset(&mut self) {
        self.inner.reset();
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
        self.inner.add_clause__(&mut lits.into_iter().map_into::<Lit>())
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
