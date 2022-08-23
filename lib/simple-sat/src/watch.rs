use std::ops::IndexMut;

use crate::cref::ClauseRef;
use crate::index_map::LitVec;
use crate::lit::Lit;
use crate::var::Var;

#[derive(Debug, Copy, Clone)]
pub struct Watcher {
    pub(crate) cref: ClauseRef,
    pub(crate) blocker: Lit,
}

#[derive(Debug)]
pub struct WatchList {
    watchlist: LitVec<Vec<Watcher>>,
}

impl WatchList {
    pub const fn new() -> Self {
        Self { watchlist: LitVec::new() }
    }

    pub fn init(&mut self, var: Var) {
        self.watchlist.init(&Lit::new(var, false));
        self.watchlist.init(&Lit::new(var, true));
    }

    pub fn lookup(&mut self, lit: Lit) -> &mut Vec<Watcher> {
        self.watchlist.index_mut(lit)
    }

    pub fn insert(&mut self, lit: Lit, watch: Watcher) {
        self.watchlist[lit].push(watch);
    }
}
