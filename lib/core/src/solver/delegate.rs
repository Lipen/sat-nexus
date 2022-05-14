use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use crate::lit::Lit;
use crate::solver::{BaseSolver, LitValue, SolveResponse, Solver};

pub struct DelegateSolver {
    inner: Box<dyn Solver>,
}

impl DelegateSolver {
    pub fn new(inner: impl Solver + 'static) -> Self {
        Self { inner: Box::new(inner) }
    }
}

impl From<Box<dyn Solver>> for DelegateSolver {
    fn from(inner: Box<dyn Solver>) -> Self {
        DelegateSolver { inner }
    }
}

impl Display for DelegateSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", tynm::type_name::<Self>(), self.inner)
    }
}

impl BaseSolver for DelegateSolver {
    fn assume_(&mut self, lit: Lit) {
        self.inner.assume_(lit)
    }

    fn value_(&self, lit: Lit) -> LitValue {
        self.inner.value_(lit)
    }

    fn add_clause_(&mut self, lits: &[Lit]) {
        self.inner.add_clause_(lits)
    }

    fn add_clause__(&mut self, lits: &mut dyn Iterator<Item = Lit>) {
        self.inner.add_clause__(lits)
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

    fn solve(&mut self) -> SolveResponse {
        self.inner.solve()
    }
}
