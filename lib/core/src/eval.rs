use std::hash::Hash;

use ndarray::{Array, Dimension};

use crate::domainvar::DomainVar;
use crate::lit::Lit;
use crate::solver::{LitValue, Solver};

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
