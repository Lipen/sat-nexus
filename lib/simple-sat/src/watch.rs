use std::ops::IndexMut;
use std::rc::Rc;

use crate::clause::Clause;
use crate::index_map::LitVec;
use crate::lit::Lit;
use crate::var::Var;

#[derive(Debug, Clone)]
pub struct Watcher {
    pub(crate) clause: Rc<Clause>,
    pub(crate) blocker: Lit,
}

#[derive(Debug)]
pub struct WatchList {
    watchlist: LitVec<Vec<Watcher>>,
}

impl WatchList {
    pub fn new() -> Self {
        Self {
            // watchlist: LitMap::new(),
            watchlist: LitVec::new(),
        }
    }

    pub fn init(&mut self, var: Var) {
        // self.watchlist.insert(Lit::new(var, false), Vec::new());
        // self.watchlist.insert(Lit::new(var, true), Vec::new());
        self.watchlist.init(Lit::new(var, false));
        self.watchlist.init(Lit::new(var, true));
    }

    pub fn lookup(&mut self, lit: Lit) -> &mut Vec<Watcher> {
        // self.watchlist.get_mut(&lit).unwrap_or_else(|| panic!("lookup of {:?} failed", lit))
        self.watchlist.index_mut(lit)
    }

    pub fn insert(&mut self, lit: Lit, watch: Watcher) {
        // println!("WatchList::insert(lit = {:?}, watch = {:?})", lit, watch);
        assert!(
            watch.clause[0] == lit || watch.clause[1] == lit,
            "watched literal must be either at index 0 or 1"
        );
        self.lookup(lit).push(watch);
    }
}
