use std::io::BufRead;
use std::mem;
use std::path::Path;
use std::ptr;
use std::time::{Duration, Instant};

// use rand::rngs::StdRng;
// use rand::{Rng, SeedableRng};
use tap::Tap;
use tracing::{debug, info};

use crate::clause::Clause;
use crate::cref::ClauseRef;
use crate::idx::VarVec;
use crate::lbool::LBool;
use crate::lit::Lit;
use crate::utils::luby;
use crate::utils::parse_dimacs_clause;
use crate::utils::read_maybe_gzip;
use crate::var::Var;
use crate::var_order::VarOrder;
use crate::watch::{WatchList, Watcher};

#[derive(Debug)]
pub struct VarData {
    reason: Option<ClauseRef>,
    level: usize,
}

#[derive(Debug)]
pub struct Solver {
    clauses: Vec<Clause>,
    watchlist: WatchList,
    assignment: VarVec<LBool>, // {var: value}
    var_data: VarVec<VarData>, // {var: {reason,level}}
    pub var_order: VarOrder,
    // seen: Vec<bool>,
    trail: Vec<Lit>,
    trail_lim: Vec<usize>,
    qhead: usize,
    ok: bool,
    next_var: u32,
    // rng: StdRng,
    // Statistics
    num_clauses: usize,
    num_learnts: usize,
    decisions: usize,
    propagations: usize,
    conflicts: usize,
    restarts: usize,
    // Timings
    pub time_search: Duration,
    pub time_propagate: Duration,
    pub time_analyze: Duration,
    pub time_backtrack: Duration,
    pub time_learn: Duration,
    pub time_restart: Duration,
    pub time_pick_decision_var: Duration,
    pub time_decision: Duration,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            clauses: vec![],
            watchlist: WatchList::new(),
            assignment: VarVec::new(),
            var_data: VarVec::new(),
            var_order: VarOrder::new(),
            // seen: Vec::new(),
            trail: vec![],
            trail_lim: vec![],
            qhead: 0,
            ok: true,
            next_var: 0,
            // rng: StdRng::seed_from_u64(42),
            num_clauses: 0,
            num_learnts: 0,
            decisions: 0,
            propagations: 0,
            conflicts: 0,
            restarts: 0,
            time_search: Duration::new(0, 0),
            time_propagate: Duration::new(0, 0),
            time_analyze: Duration::new(0, 0),
            time_backtrack: Duration::new(0, 0),
            time_learn: Duration::new(0, 0),
            time_restart: Duration::new(0, 0),
            time_pick_decision_var: Duration::new(0, 0),
            time_decision: Duration::new(0, 0),
        }
    }

    pub fn from_file<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut solver = Self::new();

        for line in read_maybe_gzip(path).unwrap().lines().flatten() {
            if line.starts_with('c') {
                // println!("Skipping comment '{}'", s);
                continue;
            } else if line.starts_with('p') {
                debug!("Skipping header '{}'", line);
                continue;
            }
            let lits = parse_dimacs_clause(&line);
            solver.add_clause(&lits);
        }

        solver
    }

    pub fn num_vars(&self) -> usize {
        self.next_var as _
    }
    pub fn num_clauses(&self) -> usize {
        self.num_clauses
        // self.clauses.iter().filter(|x| !x.learnt).count()
    }
    pub fn num_learnts(&self) -> usize {
        self.num_learnts
        // self.clauses.iter().filter(|x| x.learnt).count()
    }
    pub fn num_decisions(&self) -> usize {
        self.decisions
    }
    pub fn num_propagations(&self) -> usize {
        self.propagations
    }
    pub fn num_conflicts(&self) -> usize {
        self.conflicts
    }
    pub fn num_restarts(&self) -> usize {
        self.restarts
    }

    pub fn new_var(&mut self) -> Var {
        let var = Var(self.next_var);
        self.next_var += 1;

        // Watch
        self.watchlist.init(var);

        // Assignment
        self.assignment.push(LBool::Undef);

        // Reason/level
        self.var_data.push(VarData { reason: None, level: 0 });

        // Seen
        // self.seen.push(false);

        // VSIDS
        self.var_order.push_zero_activity();
        self.var_order.insert_var_order(var);

        // TODO: polarity, decision

        // println!("Solver::new_var -> {:?}", v);
        var
    }

    pub fn value_var(&self, var: Var) -> LBool {
        self.assignment[var]
    }
    pub fn value(&self, lit: Lit) -> LBool {
        self.value_var(lit.var()) ^ lit.negated()
    }

    pub fn var_data(&self, var: Var) -> &VarData {
        &self.var_data[var]
    }
    pub fn reason(&self, var: Var) -> Option<ClauseRef> {
        self.var_data(var).reason
    }
    pub fn level(&self, var: Var) -> usize {
        self.var_data(var).level
    }

    pub fn decision_level(&self) -> usize {
        self.trail_lim.len()
    }

    fn new_decision_level(&mut self) {
        self.trail_lim.push(self.trail.len());
    }

    fn backtrack(&mut self, level: usize) {
        let time_backtrack_start = Instant::now();
        debug!("backtrack from {} to {}", self.decision_level(), level);
        debug_assert!(level == 0 || self.decision_level() > level); // actually, assert is not needed
        if self.decision_level() > level {
            for i in (self.trail_lim[level]..self.trail.len()).rev() {
                let var = self.trail[i].var();
                self.assignment[var] = LBool::Undef;
                self.var_order.insert_var_order(var);
                // TODO: phase saving
            }
            self.qhead = self.trail_lim[level];
            self.trail.truncate(self.trail_lim[level]);
            self.trail_lim.truncate(level);
        }
        // self.qhead = std::cmp::min(self.qhead, self.trail.len())
        let time_backtrack = time_backtrack_start.elapsed();
        self.time_backtrack += time_backtrack;
    }

    pub fn add_clause(&mut self, lits: &[Lit]) -> bool {
        assert_eq!(self.decision_level(), 0);
        assert!(!lits.is_empty());

        // If the solver is already in UNSAT state, we do not need to add new clause.
        if !self.ok {
            return false;
        }

        // Note: We assume that the clause does not have duplicates or tautologies.

        // Auto-create missing variables.
        let max_var = lits.iter().map(|&lit| lit.var().0 + 1).max().unwrap() as _; // 1-based
        assert!(max_var > 0);
        for _ in (self.num_vars() + 1)..=max_var {
            self.new_var();
        }

        // TODO: handle unit clauses (better)

        let cref = self.alloc(lits.to_vec(), false);
        let clause = self.clause(cref);
        if clause.len() >= 2 {
            self.attach_clause(cref);
        } else {
            assert_eq!(clause.len(), 1);
            // info!("adding unit clause: {:?}", clause);
            if !self.enqueue(clause[0], None) {
                self.ok = false;
            }
        }
        self.ok
    }

    fn clause(&self, cref: ClauseRef) -> &Clause {
        &self.clauses[cref]
    }

    fn alloc(&mut self, lits: Vec<Lit>, learnt: bool) -> ClauseRef {
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

    fn attach_clause(&mut self, cref: ClauseRef) {
        let clause = self.clause(cref);
        debug_assert!(clause.len() >= 2, "Clause must have at least 2 literals");
        let a = clause[0];
        let b = clause[1];
        self.watchlist.insert(a, Watcher { cref, blocker: b });
        self.watchlist.insert(b, Watcher { cref, blocker: a });
    }

    pub fn solve(&mut self) -> bool {
        info!("Solver::solve()");

        // If the solver is already in UNSAT state, return early.
        if !self.ok {
            return false;
        }

        let restart_init = 100; // MiniSat: 100
        let restart_inc = 2.0; // MiniSat: 2.0

        let mut status = None;
        let mut current_restarts = 0;
        while status.is_none() {
            let restart_base = luby(restart_inc, current_restarts);
            let num_confl = (restart_base * restart_init as f64) as usize;
            let time_search_start = Instant::now();
            status = self.search(num_confl);
            current_restarts += 1;
            let time_search = time_search_start.elapsed();
            self.time_search += time_search;
            debug!("Search #{} done in {:?}", current_restarts, time_search);
        }

        self.backtrack(0);
        status.unwrap()
    }

    /// The function `search` is the main CDCL loop.
    /// It calls `propagate` to propagate the current assignment, and if a conflict
    /// is found, it calls `analyze` to analyze the conflict and learn a new clause,
    /// and then it calls `backtrack` to backtrack to a lower decision level.
    /// If no conflict is found, it calls `pick_branching_variable` to pick a new variable
    /// to branch on, and then it calls `new_decision_level` to create a new decision level,
    /// and enqueues the new decision.
    ///
    /// **Arguments:**
    ///
    /// * `num_confl`: the number of conflicts before a restart.
    ///
    /// **Returns:**
    ///
    /// [`Some(true)`][Some] if the formula is satisfiable (no more unassigned variables),
    /// [`Some(false)`][Some] if it is unsatisfiable (found a conflict on the ground level),
    /// and [`None`] if it is unknown yet (for example, a restart occurred).
    fn search(&mut self, num_confl: usize) -> Option<bool> {
        debug!("Solver::search(num_confl = {})", num_confl);
        assert!(self.ok);

        let mut current_conflicts = 0;

        // info!(". level restarts conflicts learnts clauses vars");

        // CDCL loop
        loop {
            let time_propagate_start = Instant::now();
            // let props = self.num_propagations();
            if let Some(conflict) = self.propagate().tap(|_| {
                let time_propagate = time_propagate_start.elapsed();
                self.time_propagate += time_propagate;
                // println!("Done {} props in {:?}", self.num_propagations() - props, time_propagate);
            }) {
                // Conflict
                current_conflicts += 1;
                self.conflicts += 1;

                if self.decision_level() == 0 {
                    // conflict on root level => UNSAT
                    info!("UNSAT");
                    return Some(false);
                }

                let time_analyze_start = Instant::now();
                let (lemma, backtrack_level) = self.analyze(conflict);
                let time_analyze = time_analyze_start.elapsed();
                self.time_analyze += time_analyze;
                // println!("analyzed conflict in {:?}", time_analyze);
                self.backtrack(backtrack_level);

                let time_learn_start = Instant::now();
                assert!(lemma.len() > 0);
                if lemma.len() == 1 {
                    // Learn a unit clause
                    self.unchecked_enqueue(lemma[0], None);
                    info!(
                        "unit @{} rst={} dec={} prp={} cfl={} lrn={} cls={} vrs={}",
                        self.decision_level(),
                        self.num_restarts(),
                        self.num_decisions(),
                        self.num_propagations(),
                        self.num_conflicts(),
                        self.num_learnts(),
                        self.num_clauses(),
                        self.num_vars()
                    );
                } else {
                    // Learn a clause
                    let cref = self.alloc(lemma, true);
                    self.attach_clause(cref);
                    let lemma = self.clause(cref);
                    let asserting_literal = lemma[0];
                    self.unchecked_enqueue(asserting_literal, Some(cref));
                }
                let time_learn = time_learn_start.elapsed();
                self.time_learn += time_learn;

                self.var_order.var_decay_activity();
            } else {
                // NO conflict

                // TODO: budget
                // TODO: inprocessing

                // Restart:
                if num_confl > 0 && current_conflicts >= num_confl {
                    self.restarts += 1;
                    let time_restart_start = Instant::now();
                    self.backtrack(0);
                    let time_restart = time_restart_start.elapsed();
                    self.time_restart += time_restart;
                    info!(
                        "restart @{} rst={} dec={} prp={} cfl={} lrn={} cls={} vrs={}",
                        self.decision_level(),
                        self.num_restarts(),
                        self.num_decisions(),
                        self.num_propagations(),
                        self.num_conflicts(),
                        self.num_learnts(),
                        self.num_clauses(),
                        self.num_vars()
                    );
                    // info!("Restart");
                    // info!("  vars:         {}", self.num_vars());
                    // info!("  clauses:      {}", self.num_clauses());
                    // info!("  learnts:      {}", self.num_learnts());
                    // info!("  decisions:    {}", self.num_decisions());
                    // info!("  propagations: {}", self.num_propagations());
                    // info!("  conflicts:    {}", self.num_conflicts());
                    // info!("  restarts:     {}", self.num_restarts());
                    return None;
                }

                // Make a decision:
                self.decisions += 1;
                let time_pick_decision_var_start = Instant::now();
                let time_decision_start = Instant::now();
                if let Some(var) = self.pick_branching_variable().tap(|_| {
                    let time_pick_decision_var = time_pick_decision_var_start.elapsed();
                    self.time_pick_decision_var += time_pick_decision_var;
                }) {
                    // let decision = Lit::new(var, self.rng.gen()); // random phase
                    let decision = Lit::new(var, false); // always positive phase
                    self.new_decision_level();
                    self.unchecked_enqueue(decision, None);
                    let time_decision = time_decision_start.elapsed();
                    self.time_decision += time_decision;
                    debug!(
                        "Made a decision {:?} = {}{:?} in {:?}",
                        decision,
                        if decision.negated() { "-" } else { "+" },
                        decision.var(),
                        time_decision,
                    );
                } else {
                    // SAT
                    info!("SAT");
                    return Some(true);
                }
            }
        }
    }

    fn pick_branching_variable(&mut self) -> Option<Var> {
        // Fixed-order strategy
        // for i in 0..self.num_vars() {
        //     let var = Var(i as _);
        //     if self.value_var(var).is_none() {
        //         // println!("Solver::pick_branching_variable(): found unassigned {:?} for i = {}", lit, i);
        //         return Some(var);
        //     }
        // }
        // None

        // Activity-based strategy
        self.var_order.pick_branching_variable(&self.assignment)

        // let mut next = None;
        // while next.is_none() || self.value_var(next.unwrap()).is_some() {
        //     // if let Some((var, activity)) = self.order_heap.pop() {
        //     //     debug!("next = {:?} with activity = {}", var, activity);
        //     let ref act = self.activity;
        //     if let Some(var) = self.order_heap.pop_by(|a, b| act[a] > act[b]) {
        //         next = Some(var);
        //     } else {
        //         next = None;
        //         break;
        //     }
        // }
        // next
    }

    /// If the literal is unassigned, assign it;
    /// if it's already assigned, do nothing;
    /// if it's assigned to false (conflict), return false.
    ///
    /// **Arguments:**
    ///
    /// * `lit`: The literal to be assigned.
    /// * `reason`: the reason for the assignment of lit.
    ///
    /// **Returns:**
    ///
    /// A boolean indicating whether the enqueue was successful.
    fn enqueue(&mut self, lit: Lit, reason: Option<ClauseRef>) -> bool {
        // println!("Solver::enqueue(lit = {:?}, reason = {:?})", lit, reason);
        // match self.value(lit) {
        //     None => {
        //         // TODO: inline
        //         self.unchecked_enqueue(lit, reason);
        //         true
        //     }
        //     Some(true) => {
        //         // existing consistent assignment => do nothing
        //         info!("existing consistent assignment of {:?}", lit);
        //         true
        //     }
        //     Some(false) => {
        //         // conflict
        //         false
        //     }
        // }
        match self.value(lit) {
            LBool::Undef => {
                // TODO: inline
                self.unchecked_enqueue(lit, reason);
                true
            }
            LBool::True => {
                // existing consistent assignment => do nothing
                info!("existing consistent assignment of {:?}", lit);
                true
            }
            LBool::False => {
                // conflict
                false
            }
        }
    }

    fn unchecked_enqueue(&mut self, lit: Lit, reason: Option<ClauseRef>) {
        debug_assert_eq!(self.value(lit), LBool::Undef);

        self.assignment[lit.var()] = LBool::from(!lit.negated());
        self.var_data[lit.var()] = VarData {
            reason,
            level: self.decision_level(),
        };
        self.trail.push(lit);
    }

    fn propagate(&mut self) -> Option<ClauseRef> {
        let mut conflict = None;

        #[inline]
        fn ptr_diff<T>(a: *const T, b: *const T) -> usize {
            ((b as usize) - (a as usize)) / mem::size_of::<T>()
        }

        while self.qhead < self.trail.len() {
            let p = self.trail[self.qhead];
            self.qhead += 1;
            // debug!("propagating {:?}", p);
            self.propagations += 1;

            // // Skip the propagation when the conflict was already found:
            // if conflict.is_some() {
            //     continue;
            // }

            let false_literal = !p;

            // let ws = self.watchlist.lookup(false_literal);
            // let ws = std::mem::replace(ws, Vec::with_capacity(ws.len()));

            unsafe {
                let watchers = self.watchlist.lookup(false_literal);
                let ws = watchers.as_mut_ptr();

                let begin = ws;
                let end = begin.add(watchers.len());

                let mut i = begin;
                let mut j = begin;

                // 'watches: for w in ws {
                'watches: while i < end {
                    let w = *i;
                    i = i.add(1);

                    // // Re-insert the watch, if a conflict has already been found:
                    // if conflict.is_some() {
                    //     // self.watchlist.insert(false_literal, w);
                    //     *j = w.clone();
                    //     j = j.add(1);
                    //     continue;
                    // }

                    // Try to avoid inspecting the clause:
                    if self.value(w.blocker) == LBool::True {
                        // println!("blocker {:?} is satisfied", w.blocker);
                        // self.watchlist.insert(false_literal, w);
                        *j = w;
                        j = j.add(1);
                        continue;
                    }

                    let clause = self.clause(w.cref);

                    // Make sure the false literal is at index 1:
                    if clause[0] == false_literal {
                        // println!("swapping {:?} and {:?}", w.clause[0], w.clause[1]);
                        // Rc::get_mut(&mut w.clause).unwrap().lits.swap(0, 1);
                        // FIXME: unsafe!
                        // unsafe {
                        let p = clause.lits.as_ptr() as *mut Lit;
                        ptr::swap(p, p.add(1));
                        // *p = *p.add(1);
                        // *p.add(1) = false_literal;
                        // }
                    }
                    debug_assert_eq!(clause[1], false_literal, "clause[1] must be false_literal");

                    // If the 0th literal is `true`, then the clause is already satisfied
                    // TODO: let first = w.clause[0] & w.clause[1] ^ false_literal;
                    let first = clause[0];
                    let w = Watcher {
                        cref: w.cref,
                        blocker: first,
                    };
                    if self.value(first) == LBool::True {
                        // self.watchlist.insert(false_literal, w);
                        *j = w;
                        j = j.add(1);
                        continue;
                    }

                    // Find the non-falsified literal:
                    for i in 2..clause.len() {
                        let other = clause[i];
                        if self.value(other) != LBool::False {
                            // Rc::get_mut(&mut w.clause).unwrap().lits.swap(1, i);
                            // FIXME: unsafe!
                            // unsafe {
                            let p = clause.lits.as_ptr() as *mut Lit;
                            ptr::swap(p.add(1), p.add(i));
                            // *p.add(1) = other;
                            // *p.add(i) = false_literal;
                            // }
                            self.watchlist.insert(other, w);
                            continue 'watches;
                        }
                    }

                    // self.watchlist.insert(false_literal, w);
                    *j = w;
                    j = j.add(1);
                    match self.value(first) {
                        LBool::Undef => {
                            // unit
                            // debug!("unit {:?} with reason {:?}", first, w.clause);
                            self.unchecked_enqueue(first, Some(w.cref));
                        }
                        LBool::False => {
                            // conflict
                            // debug!("conflict {:?}", w.clause);
                            conflict = Some(w.cref);
                            self.qhead = self.trail.len();
                            // TODO: copy the remaining watches here
                            while i < end {
                                *j = *i;
                                j = j.add(1);
                                i = i.add(1);
                            }
                        }
                        LBool::True => unreachable!(),
                    }
                }

                self.watchlist.lookup(false_literal).truncate(ptr_diff(begin, j));
            }
        }

        conflict
    }

    /// Returns learnt clause and backtrack level.
    fn analyze(&mut self, conflict: ClauseRef) -> (Vec<Lit>, usize) {
        debug!("analyze conflict: {:?}", conflict);

        debug_assert!(self.decision_level() > 0);

        let mut lemma = Vec::new();
        let mut seen = VarVec::from(vec![false; self.num_vars()]);
        let mut counter: u32 = 0; // number of literals in the conflicting clause on the current decision level
        let mut confl = conflict;
        let mut index = self.trail.len();

        loop {
            // let clause = self.clause(confl);
            let clause = &self.clauses[confl];

            // TODO: if conflict.learnt() { bump clause activity for 'conflict' }

            let start_index = if confl == conflict { 0 } else { 1 };
            for j in start_index..clause.len() {
                let q = clause[j];
                debug_assert_eq!(self.value(q), LBool::False);

                if !seen[q.var()] && self.level(q.var()) > 0 {
                    seen[q.var()] = true;

                    // Bump `q` variable activity:
                    self.var_order.var_bump_activity(q.var());

                    if self.level(q.var()) < self.decision_level() {
                        lemma.push(q);
                    } else {
                        debug_assert_eq!(self.level(q.var()), self.decision_level());
                        counter += 1;
                    }
                }
            }

            // Select next clause to look at:
            loop {
                index -= 1;
                if seen[self.trail[index].var()] {
                    break;
                }
            }
            let p = self.trail[index];
            seen[p.var()] = false; // TODO: why do we need to un-seen p?
            counter -= 1;
            if counter == 0 {
                // Prepend the asserting literal:
                lemma.insert(0, !p);
                break;
            }
            confl = self.reason(p.var()).unwrap();
            // debug_assert_eq!(p, clause[0]); // FIXME: failing
        }

        // FIXME: this is temporary, because we are going to use `seen` during minimization,
        //  and we cannot reuse this code *after* the minimization, because `lemma` will be shorter.
        // for lit in lemma.iter() {
        //     seen[lit.var().index()] = false;
        // }

        // TODO: minimize

        // Find the correct backtrack level:
        let bt_level = if lemma.len() == 1 {
            0
        } else {
            let mut max_i = 1;
            // Find the first literal assigned at the next-highest level:
            for i in 2..lemma.len() {
                if self.level(lemma[i].var()) > self.level(lemma[max_i].var()) {
                    max_i = i;
                }
            }
            // Swap-in this literal at index 1:
            lemma.swap(1, max_i);
            self.level(lemma[1].var())
            // explicit code:
            // let w = lemma[max_i];
            // lemma[max_i] = lemma[1];
            // lemma[1] = w;
            // self.level(w.var())
        };

        // let lemma = Rc::new(Clause { lits: lemma });
        // println!("Solver::analyze() -> ({:?}, {})", lemma, bt_level);
        (lemma, bt_level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correctness() {
        let mut solver = Solver::new();

        let tie = Lit::new(solver.new_var(), false);
        let shirt = Lit::new(solver.new_var(), false);
        println!("TIE = {:?}, SHIRT = {:?}", tie, shirt);
        solver.add_clause(&[-tie, shirt]);
        solver.add_clause(&[tie, shirt]);
        solver.add_clause(&[-tie, -shirt]);

        // Problem is satisfiable.
        let res = solver.solve();
        assert_eq!(res, true);

        // Check TIE is false, SHIRT is true.
        assert_eq!(solver.value(tie), LBool::False);
        assert_eq!(solver.value(shirt), LBool::True);

        // Force TIE to true.
        solver.add_clause(&[tie]);

        // Problem is now unsatisfiable.
        let res = solver.solve();
        assert_eq!(res, false);
    }
}
