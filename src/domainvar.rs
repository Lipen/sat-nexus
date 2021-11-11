use std::any::type_name;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use itertools::Itertools;

use crate::ipasir::LitValue;
use crate::lit::Lit;
use crate::op::encodings::Encodings;
use crate::solver::Solver;

#[derive(Debug)]
pub struct DomainVar<T> {
    map: HashMap<T, Lit>,
    domain: Vec<T>,
    lits: Vec<Lit>,
}

impl<T> DomainVar<T>
where
    T: Hash + Eq + Copy,
{
    pub fn new<S, I>(solver: &mut S, domain: I) -> Self
    where
        S: Solver,
        I: IntoIterator<Item = T>,
    {
        Self::new_onehot(solver, domain)
    }

    pub fn new_onehot<S, I>(solver: &mut S, domain: I) -> Self
    where
        S: Solver,
        I: IntoIterator<Item = T>,
    {
        let var = Self::new_bare(solver, domain);
        solver.encode_onehot(&var.lits);
        var
    }

    pub fn new_bare<S, I>(solver: &mut S, domain: I) -> Self
    where
        S: Solver,
        I: IntoIterator<Item = T>,
    {
        let domain = domain.into_iter().collect_vec();
        let lits = (0..domain.len()).map(|_| solver.new_var()).collect_vec();
        let map = domain.iter().copied().zip(lits.iter().copied()).collect();
        Self { map, domain, lits }
    }

    pub fn eq(&self, rhs: T) -> Lit {
        debug_assert!(self.map.contains_key(&rhs));
        self.map[&rhs]
    }

    pub fn neq(&self, rhs: T) -> Lit {
        -self.eq(rhs)
    }

    pub fn eval<S>(&self, solver: &S) -> T
    where
        S: Solver,
    {
        if cfg!(debug_assertions) {
            let indices = self
                .lits
                .iter()
                .positions(|&l| matches!(solver.val(l), LitValue::True))
                .collect_vec();
            // There must be exactly 1 literal which is True in the model.
            debug_assert_eq!(indices.len(), 1);
        }

        let index = self
            .lits
            .iter()
            .position(|&l| matches!(solver.val(l), LitValue::True))
            .unwrap();
        self.domain[index]
    }

    // just for tests
    pub fn reverse_domain(&mut self) {
        // Note: reverse only `domain` (and, correspondingly, keys of `map`), but not `lits`!
        self.domain.reverse();
        self.map = self
            .domain
            .iter()
            .copied()
            .zip(self.lits.iter().copied())
            .collect();
    }
}

impl<T> fmt::Display for DomainVar<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DomainVar<{}>({:?})", type_name::<T>(), self.domain)
    }
}
