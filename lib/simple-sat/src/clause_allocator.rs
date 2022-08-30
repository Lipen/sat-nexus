use std::cmp::Ordering;
use std::ops::{Index, IndexMut};

use tracing::info;

use crate::assignment::Assignment;
use crate::clause::Clause;
use crate::cref::ClauseRef;
use crate::lit::Lit;
use crate::utils::cmp_f64;

#[derive(Debug)]
pub struct ClauseAllocator {
    /// Clause database: all clauses (original and learnt).
    pub(crate) db: Vec<Clause>,
    /// Original clauses.
    clauses: Vec<ClauseRef>,
    /// Learnt clauses.
    pub(crate) learnts: Vec<ClauseRef>,
}

impl ClauseAllocator {
    pub const fn new() -> Self {
        Self {
            db: Vec::new(),
            clauses: Vec::new(),
            learnts: Vec::new(),
        }
    }
}

impl Default for ClauseAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// ca[cref]
impl Index<ClauseRef> for ClauseAllocator {
    type Output = Clause;

    fn index(&self, cref: ClauseRef) -> &Self::Output {
        self.db.index(cref.0)
    }
}

// &mut ca[cref]
impl IndexMut<ClauseRef> for ClauseAllocator {
    fn index_mut(&mut self, cref: ClauseRef) -> &mut Self::Output {
        self.db.index_mut(cref.0)
    }
}

impl ClauseAllocator {
    pub fn num_clauses(&self) -> usize {
        self.clauses.len()
    }
    pub fn num_learnts(&self) -> usize {
        self.learnts.len()
    }

    pub fn clause(&self, cref: ClauseRef) -> &Clause {
        self.index(cref)
    }
    pub fn clause_mut(&mut self, cref: ClauseRef) -> &mut Clause {
        self.index_mut(cref)
    }

    pub fn alloc(&mut self, lits: Vec<Lit>, learnt: bool) -> ClauseRef {
        let clause = Clause::new(lits, learnt);
        let cref = ClauseRef(self.db.len());
        self.db.push(clause);
        if learnt {
            self.learnts.push(cref);
        } else {
            self.clauses.push(cref);
        }
        cref
    }

    pub fn free(&mut self, cref: ClauseRef) {
        let clause = self.clause_mut(cref);
        assert!(!clause.is_deleted());
        clause.mark_deleted();
    }

    fn sort_learnts_by<F>(&mut self, f: F)
    where
        F: Fn(&Clause, &Clause) -> Ordering,
    {
        self.learnts.sort_by(|&a, &b| f(&self.db[a.0], &self.db[b.0]));
    }

    pub fn reduce(&mut self, assigns: &Assignment) {
        self.sort_learnts_by(|x, y| {
            if x.len() == 2 && y.len() == 2 {
                Ordering::Equal
            } else if x.len() == 2 {
                Ordering::Greater
            } else if y.len() == 2 {
                Ordering::Less
            } else {
                cmp_f64(x.activity(), y.activity())
            }
        });

        let cla_inc = 1.0;
        let index_lim = self.num_learnts() / 2;
        let extra_lim = cla_inc / self.num_learnts() as f64; // Remove any clause below this activity

        let learnts_before_remove = self.learnts.len();
        let mut i = 0;
        self.learnts.retain(|&cref| {
            let c = &self.db[cref.0];
            if c.is_deleted() {
                i += 1;
                return false;
            }

            let remove = c.len() > 2 && assigns.reason(c[0].var()) != Some(cref) && (i < index_lim || c.activity() < extra_lim);
            i += 1;
            if remove {
                // self.free(cref);
                // FIXME: inlining `self.free(cref)` here because of the borrow checker
                let clause = &mut self.db[cref.0];
                assert!(!clause.is_deleted());
                clause.mark_deleted();
                false
            } else {
                true
            }
        });

        let removed = learnts_before_remove - self.learnts.len();
        info!("Removed {} clauses of {}", removed, learnts_before_remove);
    }
}
