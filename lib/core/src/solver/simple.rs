use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use easy_ext::ext;

use crate::lit::Lit;

use super::types::*;

// `SimpleSolver` trait is object-safe.
const _: Option<&dyn SimpleSolver> = None;

pub trait SimpleSolver {
    fn signature(&self) -> Cow<str>;

    fn reset(&mut self);
    fn release(&mut self);

    fn num_vars(&self) -> usize;
    fn num_clauses(&self) -> usize;

    fn new_var(&mut self) -> Lit;
    fn assume(&mut self, lit: Lit);
    fn add_clause(&mut self, lits: &[Lit]);
    fn add_clause__(&mut self, lits: &mut dyn Iterator<Item = Lit>);

    fn solve(&mut self) -> SolveResponse;
    fn value(&self, lit: Lit) -> LitValue;
}

impl Display for dyn SimpleSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.signature())
    }
}

#[ext(BoxDynSimpleSolverExt)]
pub impl Box<dyn SimpleSolver> {
    fn display(&self) -> String {
        format!("{}({})", tynm::type_name::<Self>(), self.signature())
    }
}
