use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

use crate::lit::Lit;

#[derive(Debug, Clone)]
pub struct Clause {
    lits: Vec<Lit>,
    learnt: bool,
    deleted: bool,
    pub(crate) activity: f64,
}

impl Clause {
    pub const fn new(lits: Vec<Lit>, learnt: bool) -> Self {
        Self {
            lits,
            learnt,
            deleted: false,
            activity: 0.0,
        }
    }

    pub fn from_lits(lits: &[i32]) -> Self {
        let lits = lits.iter().map(|&lit| Lit::from_lit(lit)).collect();
        Self::new(lits, false)
    }

    pub fn lits(&self) -> &[Lit] {
        &self.lits
    }

    pub fn is_learnt(&self) -> bool {
        self.learnt
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    pub fn mark_deleted(&mut self) {
        self.deleted = true;
    }

    pub fn activity(&self) -> f64 {
        self.activity
    }

    pub fn len(&self) -> usize {
        self.lits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lits.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<Lit> {
        self.lits.iter()
    }
}

impl<I> Index<I> for Clause
where
    I: SliceIndex<[Lit]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.lits.index(index)
    }
}

impl<I> IndexMut<I> for Clause
where
    I: SliceIndex<[Lit]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.lits.index_mut(index)
    }
}

impl IntoIterator for Clause {
    type Item = Lit;
    type IntoIter = std::vec::IntoIter<Lit>;

    fn into_iter(self) -> Self::IntoIter {
        self.lits.into_iter()
    }
}
