use crate::clause::Clause;
use crate::cref::ClauseRef;
use crate::lit::Lit;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct ClauseAllocator {
    clauses: Vec<Clause>,
    pub(crate) num_clauses: usize,
    pub(crate) num_learnts: usize,
}

impl ClauseAllocator {
    pub fn new() -> Self {
        Self {
            clauses: Vec::new(),
            num_clauses: 0,
            num_learnts: 0,
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
        self.clauses.index(cref.0)
    }
}

// &mut ca[cref]
impl IndexMut<ClauseRef> for ClauseAllocator {
    fn index_mut(&mut self, cref: ClauseRef) -> &mut Self::Output {
        self.clauses.index_mut(cref.0)
    }
}

impl ClauseAllocator {
    pub fn alloc(&mut self, lits: Vec<Lit>, learnt: bool) -> ClauseRef {
        let clause = Clause::new(lits, learnt);
        let cref = ClauseRef(self.clauses.len());
        self.clauses.push(clause);
        if learnt {
            self.num_learnts += 1;
        } else {
            self.num_clauses += 1;
        }
        cref
    }

    pub fn clause(&self, cref: ClauseRef) -> &Clause {
        self.index(cref)
    }
    pub fn clause_mut(&mut self, cref: ClauseRef) -> &mut Clause {
        self.index_mut(cref)
    }
}
