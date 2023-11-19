use std::borrow::Cow;
use std::cmp::Ordering;

use tracing::debug;

use crate::assignment::Assignment;
use crate::clause::Clause;
use crate::clause_allocator::ClauseAllocator;
use crate::cref::ClauseRef;
use crate::lbool::LBool;
use crate::lit::Lit;
use crate::utils::cmp_f64;

#[derive(Debug)]
pub struct ClauseDatabase {
    /// Original clauses.
    clauses: Vec<ClauseRef>,
    /// Learnt clauses.
    learnts: Vec<ClauseRef>,
    // Clause activity:
    cla_decay: f64,
    cla_inc: f64,
}

const DEFAULT_CLA_DECAY: f64 = 0.999;
const DEFAULT_CLA_INC: f64 = 1.0;

impl ClauseDatabase {
    pub fn new() -> Self {
        Self {
            clauses: Vec::new(),
            learnts: Vec::new(),
            cla_decay: DEFAULT_CLA_DECAY,
            cla_inc: DEFAULT_CLA_INC,
        }
    }
}

impl Default for ClauseDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl ClauseDatabase {
    pub fn clauses(&self) -> &[ClauseRef] {
        &self.clauses
    }
    pub fn learnts(&self) -> &[ClauseRef] {
        &self.learnts
    }

    pub fn num_clauses(&self) -> usize {
        self.clauses.len()
    }
    pub fn num_learnts(&self) -> usize {
        self.learnts.len()
    }

    pub fn new_clause<'a, L>(&mut self, lits: L, learnt: bool, ca: &mut ClauseAllocator) -> ClauseRef
    where
        L: Into<Cow<'a, [Lit]>>,
    {
        let lits = lits.into().into_owned();
        let clause = Clause::new(lits, learnt);
        let cref = ca.alloc(clause);
        if learnt {
            self.learnts.push(cref);
        } else {
            self.clauses.push(cref);
        }
        cref
    }

    pub fn cla_decay_activity(&mut self) {
        self.cla_inc *= 1.0 / self.cla_decay;
    }

    pub fn cla_bump_activity(&mut self, cref: ClauseRef, ca: &mut ClauseAllocator) {
        let clause = ca.clause_mut(cref);

        if !clause.is_learnt() {
            return;
        }

        // Bump clause activity:
        clause.activity += self.cla_inc;

        // Rescale:
        if clause.activity > 1e20 {
            // Decrease the increment value:
            self.cla_inc *= 1e-20;

            // Decrease all activities:
            for &cref in self.learnts.iter() {
                ca.clause_mut(cref).activity *= 1e-20;
            }
        }
    }

    pub fn simplify(&mut self, assignment: &Assignment, ca: &mut ClauseAllocator) {
        let all_clauses = self.clauses.iter().chain(self.learnts.iter());
        for &cref in all_clauses {
            let clause = ca.clause_mut(cref);
            if clause.is_deleted() {
                continue;
            }
            match clause.contains_fixed_literal(assignment) {
                LBool::True => {
                    debug!("{:?} contains satisfied literal => deleting", clause);
                    clause.mark_deleted();
                }
                LBool::False => {
                    debug!("{:?} contains falsified literal => shrinking", clause);
                    clause.remove_falsified_literals(assignment);
                }
                LBool::Undef => {}
            }
        }
    }

    pub fn reduce(&mut self, assignment: &Assignment, ca: &mut ClauseAllocator) {
        self.simplify(assignment, ca);

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

            let remove = c.len() > 2 && assignment.reason(c[0].var()) != Some(cref) && (i < index_lim || c.activity() < extra_lim);
            i += 1;
            if remove {
                ca.free(cref);
                false
            } else {
                true
            }
        });

        let removed = learnts_before_remove - self.learnts.len();
        debug!("Removed {} clauses of {}", removed, learnts_before_remove);
    }
}
