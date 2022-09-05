use std::cmp::Ordering;

use tracing::info;

use crate::assignment::Assignment;
use crate::clause_allocator::ClauseAllocator;
use crate::cref::ClauseRef;
use crate::lit::Lit;
use crate::utils::cmp_f64;

#[derive(Debug)]
pub struct ClauseDatabase {
    /// Original clauses.
    pub(crate) clauses: Vec<ClauseRef>,
    /// Learnt clauses.
    pub(crate) learnts: Vec<ClauseRef>,
}

impl ClauseDatabase {
    pub fn new() -> Self {
        Self {
            clauses: Vec::new(),
            learnts: Vec::new(),
        }
    }
}

impl Default for ClauseDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl ClauseDatabase {
    pub fn num_clauses(&self) -> usize {
        self.clauses.len()
    }
    pub fn num_learnts(&self) -> usize {
        self.learnts.len()
    }

    pub fn add_clause(&mut self, lits: &[Lit], learnt: bool, ca: &mut ClauseAllocator) -> ClauseRef {
        let cref = ca.alloc(lits, learnt);
        if learnt {
            self.learnts.push(cref);
        } else {
            self.clauses.push(cref);
        }
        cref
    }

    pub fn reduce(&mut self, assigns: &Assignment, ca: &mut ClauseAllocator) {
        self.learnts.sort_by(|&a, &b| {
            let x = ca.clause(a);
            let y = ca.clause(b);

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
            let c = ca.clause(cref);
            if c.is_deleted() {
                i += 1;
                return false;
            }

            let remove = c.len() > 2 && assigns.reason(c[0].var()) != Some(cref) && (i < index_lim || c.activity() < extra_lim);
            i += 1;
            if remove {
                ca.free(cref);
                false
            } else {
                true
            }
        });

        let removed = learnts_before_remove - self.learnts.len();
        info!("Removed {} clauses of {}", removed, learnts_before_remove);
    }
}
