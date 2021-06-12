use std::array;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::context::Context;
use crate::ipasir::{LitValue, SolveResponse};
use crate::types::Lit;

pub trait GenericSolver {
    fn signature(&self) -> Cow<str>;

    fn reset(&mut self);
    fn release(&mut self);

    fn context(&self) -> Rc<RefCell<Context>>;

    fn num_vars(&self) -> usize;
    fn num_clauses(&self) -> usize;

    fn new_var(&mut self) -> Lit;

    fn add_clause<I, L>(&mut self, lits: I)
    where
        I: IntoIterator<Item = L>,
        L: Into<Lit>;

    // TODO: this can be removed (migrated to use `add_clause`) after Rust 1.53 (currently, it is unstable),
    //  where IntoIterator for [T;N] was finally stabilized!
    //  https://github.com/rust-lang/rust/pull/65819#issuecomment-826859927
    fn add_clause_array<L, const N: usize>(&mut self, lits: [L; N])
    where
        L: Into<Lit>,
    {
        self.add_clause(array::IntoIter::new(lits))
    }

    // fn add_clause_unsafe<I, L>(&mut self, lits: I)
    // where
    //     I: IntoIterator<Item = L>,
    //     L: TryInto<Lit>,
    //     <L as TryInto<Lit>>::Error: std::fmt::Debug,
    // {
    //     self.add_clause(lits.into_iter().map(|x| x.try_into().unwrap()))
    // }

    fn add_clause_lit<L>(&mut self, lit: L)
    where
        L: Into<Lit>;
    fn finalize_clause(&mut self);

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>;

    fn solve(&mut self) -> SolveResponse;

    fn val<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>;

    // TODO: model

    // fn add_unit_clause(&mut self, lit: Lit) {
    //     self.add_clause(&[lit]);
    // }
    // fn add_binary_clause(&mut self, lit1: Lit, lit2: Lit);
    // fn add_ternary_clause(&mut self, lit1: Lit, lit2: Lit, lit3: Lit);
}
