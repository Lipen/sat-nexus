use std::hash::Hash;

use ndarray::{Array, ArrayD, Dimension, ShapeBuilder};

use crate::domainvar::{DomainVar, SimpleDomainVar};
use crate::eval::{Eval, SimpleEval};
use crate::lit::Lit;
use crate::solver::SimpleSolver;

use super::Solver;

impl<S> SolverExt for S where S: Solver {}

pub trait SolverExt: Solver {
    fn new_var_array<Sh>(&mut self, shape: Sh) -> Array<Lit, Sh::Dim>
    where
        Sh: ShapeBuilder,
    {
        Array::from_shape_simple_fn(shape, || self.new_var())
    }

    fn new_var_array_dyn<Sh>(&mut self, shape: Sh) -> ArrayD<Lit>
    where
        Sh: ShapeBuilder,
    {
        self.new_var_array(shape).into_dyn()
    }

    fn new_var_vec(&mut self, len: usize) -> Vec<Lit> {
        (0..len).map(|_| self.new_var()).collect()
    }

    fn new_domain_var<I>(&mut self, domain: I) -> DomainVar<I::Item>
    where
        I: IntoIterator,
        I::Item: Hash + Eq + Copy,
    {
        DomainVar::new_onehot(self, domain)
    }

    fn new_domain_var_array<Sh, I, F>(&mut self, shape: Sh, mut f_domain: F) -> Array<DomainVar<I::Item>, Sh::Dim>
    where
        Sh: ShapeBuilder,
        I: IntoIterator,
        I::Item: Hash + Eq + Copy,
        F: FnMut(<Sh::Dim as Dimension>::Pattern) -> I,
    {
        Array::from_shape_fn(shape, |pat| self.new_domain_var(f_domain(pat)))
    }

    fn new_domain_var_array_dyn<Sh, I, F>(&mut self, shape: Sh, f_domain: F) -> ArrayD<DomainVar<I::Item>>
    where
        Sh: ShapeBuilder,
        I: IntoIterator,
        I::Item: Hash + Eq + Copy,
        F: FnMut(<Sh::Dim as Dimension>::Pattern) -> I,
    {
        self.new_domain_var_array(shape, f_domain).into_dyn()
    }

    fn eval<E>(&self, value: &E) -> E::Output
    where
        E: Eval,
    {
        value.eval(self)
    }
}

impl<S> SimpleSolverExt for S where S: SimpleSolver {}

pub trait SimpleSolverExt: SimpleSolver + Sized {
    fn new_var_array<Sh>(&mut self, shape: Sh) -> Array<Lit, Sh::Dim>
    where
        Sh: ShapeBuilder,
    {
        Array::from_shape_simple_fn(shape, || self.new_var())
    }

    fn new_var_array_dyn<Sh>(&mut self, shape: Sh) -> ArrayD<Lit>
    where
        Sh: ShapeBuilder,
    {
        self.new_var_array(shape).into_dyn()
    }

    fn new_var_vec(&mut self, len: usize) -> Vec<Lit> {
        (0..len).map(|_| self.new_var()).collect()
    }

    fn new_domain_var<I>(&mut self, domain: I) -> SimpleDomainVar<I::Item>
    where
        I: IntoIterator,
        I::Item: Hash + Eq + Copy,
    {
        SimpleDomainVar::new_onehot(self, domain)
    }

    fn new_domain_var_array<Sh, I, F>(&mut self, shape: Sh, mut f_domain: F) -> Array<SimpleDomainVar<I::Item>, Sh::Dim>
    where
        Sh: ShapeBuilder,
        I: IntoIterator,
        I::Item: Hash + Eq + Copy,
        F: FnMut(<Sh::Dim as Dimension>::Pattern) -> I,
    {
        Array::from_shape_fn(shape, |pat| self.new_domain_var(f_domain(pat)))
    }

    fn new_domain_var_array_dyn<Sh, I, F>(&mut self, shape: Sh, f_domain: F) -> ArrayD<SimpleDomainVar<I::Item>>
    where
        Sh: ShapeBuilder,
        I: IntoIterator,
        I::Item: Hash + Eq + Copy,
        F: FnMut(<Sh::Dim as Dimension>::Pattern) -> I,
    {
        self.new_domain_var_array(shape, f_domain).into_dyn()
    }

    fn eval<E>(&self, value: &E) -> E::Output
    where
        E: SimpleEval,
    {
        value.eval(self)
    }
}
