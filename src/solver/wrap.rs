use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;
use ndarray::ArrayD;

use crate::context::Context;
use crate::ipasir::solver::IpasirSolver;
use crate::ipasir::{Ipasir, LitValue, SolveResponse};
use crate::solver::GenericSolver;
use crate::types::{DomainVar, Lit};

#[derive(Debug)]
pub struct WrappedIpasirSolver<S>
where
    S: Ipasir,
{
    inner: S,
    context: Rc<RefCell<Context>>,
    nvars: usize,
    nclauses: usize,
    tmp_clause: Vec<Lit>,
}

impl WrappedIpasirSolver<IpasirSolver> {
    pub fn new(inner: IpasirSolver) -> Self {
        Self {
            inner,
            context: Rc::new(RefCell::new(Context::new())),
            nvars: 0,
            nclauses: 0,
            tmp_clause: Vec::new(),
        }
    }

    pub fn new_cadical() -> Self {
        Self::new(IpasirSolver::new_cadical())
    }
    pub fn new_minisat() -> Self {
        Self::new(IpasirSolver::new_minisat())
    }
    pub fn new_glucose() -> Self {
        Self::new(IpasirSolver::new_glucose())
    }

    pub fn new_domain_var<T, I>(&mut self, domain: I) -> DomainVar<T>
    where
        T: std::hash::Hash + Eq + Copy,
        I: IntoIterator<Item = T>,
    {
        DomainVar::new(self, domain)
    }

    pub fn new_array<A, F>(shape: &[usize], f: F) -> ArrayD<A>
    where
        F: FnMut() -> A,
    {
        ArrayD::from_shape_simple_fn(shape, f)
    }

    pub fn new_var_array(&mut self, shape: &[usize]) -> ArrayD<Lit> {
        ArrayD::from_shape_simple_fn(shape, || self.new_var())
    }

    pub fn new_var_vec(&mut self, len: usize) -> Vec<Lit> {
        (0..len).map(|_| self.new_var()).collect()
    }

    pub fn add_unit<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.nclauses += 1;
        self.inner.add(lit.into().get());
        self.inner.add(0);
    }
}

impl From<IpasirSolver> for WrappedIpasirSolver<IpasirSolver> {
    fn from(inner: IpasirSolver) -> Self {
        WrappedIpasirSolver::new(inner)
    }
}

impl fmt::Display for WrappedIpasirSolver<IpasirSolver> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WrappedSolver({})", self.signature())
    }
}

impl GenericSolver for WrappedIpasirSolver<IpasirSolver> {
    fn signature(&self) -> Cow<str> {
        self.inner.signature().into()
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
    fn release(&mut self) {
        self.inner.release();
    }

    fn context(&self) -> Rc<RefCell<Context>> {
        Rc::clone(&self.context)
    }

    fn num_vars(&self) -> usize {
        self.nvars
    }
    fn num_clauses(&self) -> usize {
        self.nclauses
    }

    fn new_var(&mut self) -> Lit {
        self.nvars += 1;
        Lit::new(self.nvars as i32)
    }

    fn add_clause<I, L>(&mut self, lits: I)
    where
        I: IntoIterator<Item = L>,
        L: Into<Lit>,
    {
        self.nclauses += 1;
        self.inner.add_clause(lits.into_iter().map_into::<Lit>());
    }

    fn add_clause_lit<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.tmp_clause.push(lit.into());
    }

    fn finalize_clause(&mut self) {
        let lits = std::mem::take(&mut self.tmp_clause);
        self.add_clause(lits);
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.inner.assume(lit.into().into());
    }

    fn solve(&mut self) -> SolveResponse {
        self.inner
            .solve()
            .unwrap_or_else(|e| panic!("Could not solve: {}", e))
    }

    fn val<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        self.inner
            .val(lit.into().into())
            .expect("Could not get literal value")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_solver() -> color_eyre::Result<()> {
        let mut solver = WrappedIpasirSolver::new_cadical();
        assert!(solver.signature().contains("cadical"));

        // Adding [(1 or 2) and (3 or 4) and not(1 and 2) and not(3 and 4)]
        solver.add_clause([1, 2]);
        solver.add_clause([3, 4]);
        solver.add_clause([-1, -2]);
        solver.add_clause([-3, -4]);

        // Problem is satisfiable
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        // Assuming both 1 and 2 to be true
        solver.assume(1);
        solver.assume(2);
        // Problem is unsatisfiable under assumptions
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Unsat);

        // `solve` resets assumptions, so calling it again should produce SAT
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        Ok(())
    }
}
