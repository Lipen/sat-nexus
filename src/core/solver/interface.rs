use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use ndarray::{Array, ArrayD, Dimension, ShapeBuilder};

use crate::context::Context;
use crate::core::domainvar::DomainVar;
use crate::core::lit::Lit;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SolveResponse {
    Sat,
    Unsat,
    Unknown,
}

#[derive(Debug, Copy, Clone)]
pub enum LitValue {
    True,
    False,
    DontCare,
}

impl LitValue {
    pub fn bool(&self) -> bool {
        use LitValue::*;
        match self {
            True => true,
            False => false,
            DontCare => panic!("DontCare can't be converted to bool!"),
        }
    }
}

impl fmt::Display for LitValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LitValue::*;
        match self {
            True => write!(f, "1"),
            False => write!(f, "0"),
            DontCare => write!(f, "X"),
        }
    }
}

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
        DomainVar::new_onehot(self, domain)
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

    fn add_unit<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.add_clause([lit]);
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>;

    fn solve(&mut self) -> SolveResponse;

    fn val<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>;

    // TODO: model

    fn eval<E>(&self, value: &E) -> E::Output
    where
        Self: Sized,
        E: Eval,
    {
        value.eval(self)
    }
}

pub trait Eval {
    type Output;

    fn eval<S>(&self, solver: &S) -> Self::Output
    where
        S: Solver;
}

impl Eval for Lit {
    type Output = LitValue;

    fn eval<S>(&self, solver: &S) -> Self::Output
    where
        S: Solver,
    {
        solver.val(self)
    }
}

impl<T> Eval for DomainVar<T>
where
    T: Hash + Eq + Copy,
{
    type Output = T;

    fn eval<S>(&self, solver: &S) -> Self::Output
    where
        S: Solver,
    {
        DomainVar::eval(self, solver)
    }
}

impl<T, E, D> Eval for Array<E, D>
where
    E: Eval<Output = T>,
    D: Dimension,
{
    type Output = Array<T, D>;

    fn eval<S>(&self, solver: &S) -> Self::Output
    where
        S: Solver,
    {
        self.map(|v| v.eval(solver))
    }
}
