use std::ops::{Index, IndexMut};

use crate::clause::Clause;
use crate::cref::ClauseRef;
use crate::lit::Lit;

#[derive(Debug)]
pub struct ClauseAllocator {
    /// Clause database: all clauses (original and learnt).
    db: Vec<Clause>,
    /// Original clauses.
    clauses: Vec<ClauseRef>,
    /// Learnt clauses.
    learnts: Vec<ClauseRef>,
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
}
