use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

use tracing::debug;

use crate::assignment::Assignment;
use crate::lbool::LBool;
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

    pub(crate) fn is_satisfied(&self, assignment: &Assignment) -> bool {
        self.lits.iter().any(|&lit| assignment.value(lit) == LBool::True)
    }

    /// **Returns:**
    ///
    /// - [`LBool::True`], if clause contains root-level satisfied literal,
    /// - [`LBool::False`], if clause contains root-level falsified literal,
    /// - [`LBool::Undef`], if clause does not contain any fixed literal.
    pub(crate) fn contains_fixed_literal(&self, assignment: &Assignment) -> LBool {
        let mut num_satisfied: usize = 0;
        let mut num_falsified: usize = 0;
        for &lit in self.lits.iter() {
            match assignment.fixed(lit) {
                LBool::True => {
                    debug!("Root-level satisfied literal {:?} in {:?}", lit, self);
                    num_satisfied += 1
                }
                LBool::False => {
                    debug!("Root-level falsified literal {:?} in {:?}", lit, self);
                    num_falsified += 1;
                }
                LBool::Undef => {}
            }
        }
        if num_satisfied > 0 {
            LBool::True
        } else if num_falsified > 0 {
            LBool::False
        } else {
            LBool::Undef
        }
    }

    pub(crate) fn remove_falsified_literals(&mut self, assignment: &Assignment) {
        assert_eq!(assignment.value(self.lits[0]), LBool::Undef);
        assert_eq!(assignment.value(self.lits[1]), LBool::Undef);
        let mut i = 2;
        let mut j = self.lits.len();
        while i < j {
            if assignment.fixed(self.lits[i]) == LBool::False {
                self.lits.swap_remove(i);
                j -= 1;
            } else {
                i += 1;
            }
        }
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
