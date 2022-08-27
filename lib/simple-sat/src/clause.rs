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

    pub fn is_empty(&self) -> bool {
        self.lits.is_empty()
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
