use std::borrow::Cow;
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

use ndarray::{Array, ArrayD, Dimension, ShapeBuilder};

use crate::context::Context;
use crate::domainvar::DomainVar;
use crate::ipasir::{LitValue, SolveResponse};
use crate::lit::Lit;

pub trait Solver {
    fn signature(&self) -> Cow<str>;

    fn reset(&mut self);
    fn release(&mut self);

    fn context(&self) -> Rc<RefCell<Context>>;

    fn num_vars(&self) -> usize;
    fn num_clauses(&self) -> usize;

    fn new_var(&mut self) -> Lit;

    fn new_var_array<D, Sh>(&mut self, shape: Sh) -> Array<Lit, D>
    where
        D: Dimension,
        Sh: ShapeBuilder<Dim = D>,
    {
        Array::from_shape_simple_fn(shape, || self.new_var())
    }

    fn new_var_array_dyn<D, Sh>(&mut self, shape: Sh) -> ArrayD<Lit>
    where
        D: Dimension,
        Sh: ShapeBuilder<Dim = D>,
    {
        self.new_var_array(shape).into_dyn()
    }

    fn new_var_vec(&mut self, len: usize) -> Vec<Lit> {
        (0..len).map(|_| self.new_var()).collect()
    }

    fn new_domain_var<T, I>(&mut self, domain: I) -> DomainVar<T>
    where
        Self: Sized,
        T: Hash + Eq + Copy,
        I: IntoIterator<Item = T>,
    {
        DomainVar::new(self, domain)
    }

    fn new_domain_var_array<T, I, D, Sh, F>(
        &mut self,
        shape: Sh,
        mut f_domain: F,
    ) -> Array<DomainVar<T>, D>
    where
        Self: Sized,
        T: Hash + Eq + Copy,
        I: IntoIterator<Item = T>,
        D: Dimension,
        Sh: ShapeBuilder<Dim = D>,
        F: FnMut(D::Pattern) -> I,
    {
        Array::from_shape_fn(shape, |pat| self.new_domain_var(f_domain(pat)))
    }

    fn new_domain_var_array_dyn<T, I, D, Sh, F>(
        &mut self,
        shape: Sh,
        f_domain: F,
    ) -> ArrayD<DomainVar<T>>
    where
        Self: Sized,
        T: Hash + Eq + Copy,
        I: IntoIterator<Item = T>,
        D: Dimension,
        Sh: ShapeBuilder<Dim = D>,
        F: FnMut(D::Pattern) -> I,
    {
        self.new_domain_var_array(shape, f_domain).into_dyn()
    }

    fn add_clause<I, L>(&mut self, lits: I)
    where
        I: IntoIterator<Item = L>,
        L: Into<Lit>;

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
