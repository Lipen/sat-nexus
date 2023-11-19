use std::ops::{Index, IndexMut};

use crate::clause::Clause;
use crate::cref::ClauseRef;
use crate::lit::Lit;

#[derive(Debug)]
pub struct ClauseAllocator {
    /// All clauses.
    arena: Vec<Clause>,
}

impl ClauseAllocator {
    pub const fn new() -> Self {
        Self { arena: Vec::new() }
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
        self.arena.index(cref.0)
    }
}

// &mut ca[cref]
impl IndexMut<ClauseRef> for ClauseAllocator {
    fn index_mut(&mut self, cref: ClauseRef) -> &mut Self::Output {
        self.arena.index_mut(cref.0)
    }
}

impl ClauseAllocator {
    pub fn clause(&self, cref: ClauseRef) -> &Clause {
        self.index(cref)
    }
    pub fn clause_mut(&mut self, cref: ClauseRef) -> &mut Clause {
        self.index_mut(cref)
    }

    pub fn alloc(&mut self, lits: Vec<Lit>, learnt: bool) -> ClauseRef {
        let clause = Clause::new(lits, learnt);
        let cref = ClauseRef(self.arena.len());
        self.arena.push(clause);
        cref
    }

    pub fn free(&mut self, cref: ClauseRef) {
        let clause = self.clause_mut(cref);
        assert!(!clause.is_deleted());
        clause.mark_deleted();
    }
}
