#![allow(dead_code)]

use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

use crate::lit::Lit;

#[derive(Debug, Clone)]
pub struct Clause {
    pub(crate) lits: Vec<Lit>,
    pub(crate) learnt: bool,
}

impl Clause {
    pub const fn new(lits: Vec<Lit>, learnt: bool) -> Self {
        Self { lits, learnt }
    }

    pub fn from_lits(lits: &[i32]) -> Self {
        let lits = lits.iter().map(|&lit| Lit::from_lit(lit)).collect();
        Self::new(lits, false)
    }

    pub fn len(&self) -> usize {
        self.lits.len()
    }
}

impl Index<usize> for Clause {
    type Output = Lit;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lits[index]
    }
}

impl IndexMut<usize> for Clause {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lits[index]
    }
}

impl IntoIterator for Clause {
    type Item = Lit;
    type IntoIter = std::vec::IntoIter<Lit>;

    fn into_iter(self) -> Self::IntoIter {
        self.lits.into_iter()
    }
}
