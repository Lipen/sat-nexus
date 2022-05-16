use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

use crate::lit::Lit;
use crate::solver::simple::SimpleSolver;
use crate::solver::{LitValue, SolveResponse, Solver};

/// Implementation of [SimpleSolver] that wraps the [Solver] instance.
#[derive(Debug)]
pub struct WrapSolver<S>
where
    S: Solver,
{
    inner: S,
}

impl<S> WrapSolver<S>
where
    S: Solver,
{
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Display for WrapSolver<S>
where
    S: Solver + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", tynm::type_name::<Self>(), self.inner)
    }
}

impl<S> SimpleSolver for WrapSolver<S>
where
    S: Solver,
{
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

    fn assume(&mut self, lit: Lit) {
        self.inner.assume(lit)
    }

    fn add_clause(&mut self, lits: &[Lit]) {
        self.inner.add_clause_(lits)
    }

    fn add_clause__(&mut self, lits: &mut dyn Iterator<Item = Lit>) {
        self.inner.add_clause(lits)
    }

    fn solve(&mut self) -> SolveResponse {
        self.inner.solve()
    }

    fn value(&self, lit: Lit) -> LitValue {
        self.inner.value(lit)
    }
}

impl<S> From<S> for Box<dyn SimpleSolver>
where
    S: Solver + 'static,
{
    fn from(inner: S) -> Self {
        Box::new(WrapSolver::new(inner))
    }
}
