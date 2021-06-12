#![allow(dead_code)]

use std::any::type_name;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::ops::Neg;

use itertools::Itertools;

use crate::ipasir::Lit as IpasirLit;
use crate::ipasir::LitValue;
use crate::op::encodings::Encodings;
use crate::solver::GenericSolver;

#[derive(Debug, Copy, Clone)]
pub struct Lit(i32);

impl Lit {
    pub fn new(val: i32) -> Self {
        debug_assert!(val != 0);
        Lit(val)
    }

    pub fn get(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for Lit {
    fn from(val: i32) -> Self {
        Self::new(val)
    }
}

impl From<&i32> for Lit {
    fn from(val: &i32) -> Self {
        Self::from(*val)
    }
}

impl From<usize> for Lit {
    fn from(val: usize) -> Self {
        Self::from(val as i32)
    }
}

// Into<i32>
impl From<Lit> for i32 {
    fn from(lit: Lit) -> Self {
        lit.0
    }
}

impl From<IpasirLit> for Lit {
    fn from(lit: IpasirLit) -> Self {
        Self::new(lit.into())
    }
}

// Into<IpasirLit>
impl From<Lit> for IpasirLit {
    fn from(lit: Lit) -> Self {
        unsafe { IpasirLit::new_unchecked(lit.0) }
    }
}

impl Neg for Lit {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from(-self.0)
    }
}

// #[derive(Debug)]
// pub struct IntVar {
//     domain: Vec<i32>,
//     lits: Vec<Lit>,
// }
//
// impl IntVar {
//     pub fn new<S, I>(solver: &mut S, domain: I) -> Self
//     where
//         S: GenericSolver,
//         I: IntoIterator<Item = i32>,
//     {
//         Self::new_onehot(solver, domain)
//     }
//
//     pub fn new_bare<S, I>(solver: &mut S, domain: I) -> Self
//     where
//         S: GenericSolver,
//         I: IntoIterator<Item = i32>,
//     {
//         let domain: Vec<i32> = domain.into_iter().collect();
//         let lits: Vec<Lit> = (0..domain.len()).map(|_| solver.new_var()).collect();
//         Self { domain, lits }
//     }
//
//     pub fn new_onehot<S, I>(solver: &mut S, domain: I) -> Self
//     where
//         S: GenericSolver,
//         I: IntoIterator<Item = i32>,
//     {
//         let var = Self::new_bare(solver, domain);
//         encode_onehot(solver, &var.lits);
//         var
//     }
//
//     pub fn eq(&self, rhs: i32) -> Lit {
//         debug_assert!(self.domain.contains(&rhs));
//         let index = self.domain.iter().position(|&x| x == rhs).unwrap();
//         self.lits[index]
//     }
//
//     pub fn ne(&self, rhs: i32) -> Lit {
//         -self.eq(rhs)
//     }
//
//     pub fn eval<S>(&self, solver: &S) -> i32
//     where
//         S: GenericSolver,
//     {
//         let index = self
//             .lits
//             .iter()
//             .position(|&x| matches!(solver.val(x), LitValue::True))
//             .unwrap();
//         self.domain[index]
//     }
// }
//
// impl fmt::Display for IntVar {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "IntVar({:?})", self.domain)
//     }
// }

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
        S: GenericSolver,
        I: IntoIterator<Item = T>,
    {
        Self::new_onehot(solver, domain)
    }

    pub fn new_onehot<S, I>(solver: &mut S, domain: I) -> Self
    where
        S: GenericSolver,
        I: IntoIterator<Item = T>,
    {
        let var = Self::new_bare(solver, domain);
        solver.encode_onehot(&var.lits);
        var
    }

    pub fn new_bare<S, I>(solver: &mut S, domain: I) -> Self
    where
        S: GenericSolver,
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

    pub fn ne(&self, rhs: T) -> Lit {
        -self.eq(rhs)
    }

    pub fn eval<S>(&self, solver: &S) -> T
    where
        S: GenericSolver,
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

pub struct Clause {
    pub lits: Vec<Lit>,
}

impl Clause {
    pub fn new(lits: Vec<Lit>) -> Self {
        Clause { lits }
    }
}

impl From<Vec<Lit>> for Clause {
    fn from(value: Vec<Lit>) -> Self {
        Self::new(value)
    }
}

impl From<&[Lit]> for Clause {
    fn from(value: &[Lit]) -> Self {
        Self::new(value.to_vec())
    }
}

impl IntoIterator for Clause {
    type Item = Lit;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.lits.into_iter()
    }
}
