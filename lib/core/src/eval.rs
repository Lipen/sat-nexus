use std::hash::Hash;

use ndarray::{Array, Dimension};

use crate::domainvar::{DomainVar, SimpleDomainVar};
use crate::lit::Lit;
use crate::solver::{LitValue, SimpleSolver, Solver};

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
        solver.value(self)
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

// ===============

pub trait SimpleEval {
    type Output;

    fn eval<S>(&self, solver: &S) -> Self::Output
    where
        S: SimpleSolver;
}

impl SimpleEval for Lit {
    type Output = LitValue;

    fn eval<S>(&self, solver: &S) -> Self::Output
    where
        S: SimpleSolver,
    {
        solver.value(*self)
    }
}

impl<T> SimpleEval for SimpleDomainVar<T>
where
    T: Hash + Eq + Copy,
{
    type Output = T;

    fn eval<S>(&self, solver: &S) -> Self::Output
    where
        S: SimpleSolver,
    {
        SimpleDomainVar::eval(self, solver)
    }
}

impl<T, E, D> SimpleEval for Array<E, D>
where
    E: SimpleEval<Output = T>,
    D: Dimension,
{
    type Output = Array<T, D>;

    fn eval<S>(&self, solver: &S) -> Self::Output
    where
        S: SimpleSolver,
    {
        self.map(|v| v.eval(solver))
    }
}
