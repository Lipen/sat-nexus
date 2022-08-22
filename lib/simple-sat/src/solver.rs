use std::cmp::Ordering;
use std::mem;
use std::path::Path;
use std::ptr;
use std::rc::Rc;
use std::time::{Duration, Instant};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tap::Tap;
use tracing::{debug, info};

use crate::clause::Clause;
use crate::index_map::{VarHeap, VarMap, VarVec};
use crate::lbool::LBool;
use crate::lit::Lit;
use crate::utils::luby;
use crate::utils::parse_dimacs_clause;
use crate::utils::read_lines;
use crate::var::Var;
use crate::watch::{WatchList, Watcher};

#[derive(Debug, Clone)]
pub struct VarData {
    reason: Option<Rc<Clause>>,
    level: usize,
}

#[derive(Debug)]
pub struct Solver {
    clauses: Vec<Rc<Clause>>,
    learnts: Vec<Rc<Clause>>,
    watchlist: WatchList,
    // assignment: VarMap<bool>,  // {var: value}
    assignment: VarVec<LBool>, // {var: value}
    var_data: VarMap<VarData>, // {var: {reason,level}}
    // seen: Vec<bool>,
    trail: Vec<Lit>,
    trail_lim: Vec<usize>,
    qhead: usize,
    ok: bool,
    next_var: u32,
    rng: StdRng,
    // Statistics
    decisions: usize,
    propagations: usize,
    conflicts: usize,
    restarts: usize,
    // VSIDS
    activity: VarMap<f64>,
    // order_heap: PriorityQueue<Var, NotNan<f64>>, // max-heap ordered by activity
    order_heap: VarHeap,
    var_decay: f64,
    var_inc: f64,
    // Timings
    pub time_search: Duration,
    pub time_propagate: Duration,
    pub time_analyze: Duration,
    pub time_backtrack: Duration,
    pub time_learn: Duration,
    pub time_restart: Duration,
    pub time_pick_decision_var: Duration,
    pub time_decision: Duration,
    pub time_insert_var_order: Duration,
    pub num_insert_var_order: usize,
    pub time_update_var_order: Duration,
    pub num_update_var_order: usize,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            clauses: vec![],
            learnts: vec![],
            watchlist: WatchList::new(),
            // assignment: VarMap::new(),
            assignment: VarVec::new(),
            var_data: VarMap::new(),
            // seen: Vec::new(),
            trail: vec![],
            trail_lim: vec![],
            qhead: 0,
            ok: true,
            next_var: 0,
            rng: StdRng::seed_from_u64(42),
            decisions: 0,
            propagations: 0,
            conflicts: 0,
            restarts: 0,
            activity: VarMap::new(),
            // order_heap: PriorityQueue::new(),
            order_heap: VarHeap::new(),
            var_decay: 0.95,
            var_inc: 1.0,
            time_search: Duration::new(0, 0),
            time_propagate: Duration::new(0, 0),
            time_analyze: Duration::new(0, 0),
            time_backtrack: Duration::new(0, 0),
            time_learn: Duration::new(0, 0),
            time_restart: Duration::new(0, 0),
            time_pick_decision_var: Duration::new(0, 0),
            time_decision: Duration::new(0, 0),
            time_insert_var_order: Duration::new(0, 0),
            num_insert_var_order: 0,
            time_update_var_order: Duration::new(0, 0),
            num_update_var_order: 0,
        }
    }

    pub fn from_file(path: &Path) -> Self {
        let mut solver = Self::new();

        let lines = read_lines(path).unwrap();
        for line in lines {
            if let Ok(s) = line {
                if s.starts_with("c") {
                    // println!("Skipping comment '{}'", s);
                    continue;
                } else if s.starts_with("p") {
                    println!("Skipping header '{}'", s);
                    continue;
                }
                let lits = parse_dimacs_clause(&s);
                solver.add_clause(&lits);
            }
        }

        solver
    }

    pub fn num_vars(&self) -> usize {
        self.next_var as _
    }
    pub fn num_clauses(&self) -> usize {
        self.clauses.len()
    }
    pub fn num_learnts(&self) -> usize {
        self.learnts.len()
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
        self.var_data.insert(var, VarData { reason: None, level: 0 });

        // Seen
        // self.seen.push(false);

        // VSIDS
        self.activity.insert(var, 0.0);
        self.insert_var_order(var);

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
    // pub fn value_var(&self, var: impl Borrow<Var>) -> Option<bool> {
    //     self.assignment.get(var).copied()
    // }
    // pub fn value(&self, lit: Lit) -> Option<bool> {
    //     self.value_var(lit.var()).map(|v| v ^ lit.negated())
    // }

    pub fn var_data(&self, var: Var) -> &VarData {
        &self.var_data[var]
    }
    pub fn reason(&self, var: Var) -> Option<Rc<Clause>> {
        self.var_data(var).reason.as_ref().map(|c| Rc::clone(c))
    }
    pub fn level(&self, var: Var) -> usize {
        self.var_data(var).level
    }

    fn insert_var_order(&mut self, var: Var) {
        let time_insert_var_order_start = Instant::now();

        // self.order_heap.push(var, NotNan::new(self.activity(var)).unwrap());
        if !self.order_heap.contains(&var) {
            let ref act = self.activity;
            // self.order_heap.insert_by(var, |a, b| act[a] > act[b]);
            self.order_heap.insert_by(var, |a, b| match act[a].total_cmp(&act[b]) {
                Ordering::Less => false,
                Ordering::Equal => a.0 < b.0,
                Ordering::Greater => true,
            });
        }

        let time_insert_var_order = time_insert_var_order_start.elapsed();
        self.time_insert_var_order += time_insert_var_order;
        self.num_insert_var_order += 1;
    }
    fn update_var_order(&mut self, var: Var) {
        let time_update_var_order_start = Instant::now();

        let ref act = self.activity;
        self.order_heap.update_by(var, |a, b| act[a] > act[b]);
        // self.order_heap.update_by(var, |a, b| match act[a].total_cmp(&act[b]) {
        //     Ordering::Less => false,
        //     Ordering::Equal => a.0 < b.0,
        //     Ordering::Greater => true,
        // });

        let time_update_var_order = time_update_var_order_start.elapsed();
        self.time_update_var_order += time_update_var_order;
        self.num_update_var_order += 1;
    }
    fn var_decay_activity(&mut self) {
        self.var_inc /= self.var_decay;
    }
    fn var_bump_activity(&mut self, var: Var) {
        let new = self.activity[var] + self.var_inc;
        self.activity[var] = new;

        // Rescale large activities, if necessary:
        if new > 1e100 {
            self.var_rescale_activity();
        }

        // Update `var` in heap:
        if self.order_heap.contains(&var) {
            self.update_var_order(var);
            // let ref act = self.activity;
            // self.order_heap.decrease_by(var, |a, b| act[a] > act[b]);
            // self.order_heap.update_by(var, |a, b| act[a] > act[b]);
            // self.order_heap.update_by(var, |a, b| match act[a].total_cmp(&act[b]) {
            //     Ordering::Less => false,
            //     Ordering::Equal => a.0 < b.0,
            //     Ordering::Greater => true,
            // });
        }
    }
    fn var_rescale_activity(&mut self) {
        info!("Rescaling activity");
        // Decrease the increment value:
        // self.var_inc *= 1e-100;
        // Decrease all activities:
        for (_, a) in self.activity.iter_mut() {
            // *a *= 1e-100;
            *a /= self.var_inc;
        }
        // Decrease the increment value:
        self.var_inc = 1.0;
        // // Rebuild the heap:
        // for (v, a) in self.order_heap.iter_mut() {
        //     *a = NotNan::new(self.activity[v]).unwrap();
        // }
    }

    pub fn decision_level(&self) -> usize {
        self.trail_lim.len()
    }

    fn new_decision_level(&mut self) {
        self.trail_lim.push(self.trail.len());
    }

    fn backtrack(&mut self, level: usize) {
        debug!("backtrack from {} to {}", self.decision_level(), level);
        if level > 0 {
            assert!(self.decision_level() > level); // actually, assert is not needed
        }
        if self.decision_level() > level {
            for i in (self.trail_lim[level]..self.trail.len()).rev() {
                let var = self.trail[i].var();
                // self.assignment.remove(var);
                self.assignment[var] = LBool::Undef;
                self.insert_var_order(var);
                // TODO: phase saving
            }
            self.qhead = self.trail_lim[level];
            self.trail.truncate(self.trail_lim[level]);
            self.trail_lim.truncate(level);
        }
        // self.qhead = std::cmp::min(self.qhead, self.trail.len())
    }

    pub fn add_clause(&mut self, lits: &[Lit]) -> bool {
        // println!(
        //     "Solver::add_clause(lits = {:?})",
        //     lits.iter().map(|lit| lit.external_lit()).collect::<Vec<_>>()
        // );

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

        let clause = Clause::new(lits.to_vec());
        let clause = Rc::new(clause);
        self.clauses.push(Rc::clone(&clause));
        if clause.size() >= 2 {
            self.attach_clause(Rc::clone(&clause));
        } else {
            assert_eq!(clause.size(), 1);
            // info!("adding unit clause: {:?}", clause);
            if !self.enqueue(clause[0], None) {
                self.ok = false;
            }
        }
        self.ok
    }

    fn attach_clause(&mut self, clause: Rc<Clause>) {
        let lits = &clause.lits;
        assert!(lits.len() >= 2, "Clause must have at least 2 literals");
        self.watchlist.insert(
            lits[0],
            Watcher {
                clause: Rc::clone(&clause),
                blocker: lits[1],
            },
        );
        self.watchlist.insert(
            lits[1],
            Watcher {
                clause: Rc::clone(&clause),
                blocker: lits[0],
            },
        );
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
            println!("Search #{} done in {:?}", current_restarts, time_search);
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
    /// [`Some(true)`][Some] if the formula is satisfiable (no more unassigned variables), [`Some(false)`][Some] if it is unsatisfiable (found a conflict on the ground level), and [`None`] if it is unknown yet (for example, a restart occurred).
    fn search(&mut self, num_confl: usize) -> Option<bool> {
        info!("Solver::search(num_confl = {})", num_confl);
        assert!(self.ok);

        let mut current_conflicts = 0;

        info!(". level restarts conflicts learnts clauses vars");

        // CDCL loop
        let mut loop_index = 0;
        loop {
            loop_index += 1;
            // debug!("***** CDCL loop #{} *****", loop_index);
            // debug!("clauses: {}", self.clauses.len());
            // debug!("learnts: {}", self.learnts.len());
            // debug!("assignment = {:?}", self.assignment);
            // debug!("trail = {:?}", self.trail);
            // debug!("trail_lim = {:?}", self.trail_lim);
            // debug!("decision_level = {}", self.decision_level());

            // if loop_index % 100_000 == 0 {
            //     info!("=== CDCL loop #{}", loop_index);
            //     info!("  vars:         {}", self.num_vars());
            //     info!("  clauses:      {}", self.num_clauses());
            //     info!("  learnts:      {}", self.num_learnts());
            //     info!("  decisions:    {}", self.num_decisions());
            //     info!("  propagations: {}", self.num_propagations());
            //     info!("  conflicts:    {}", self.num_conflicts());
            //     info!("  restarts:     {}", self.num_restarts());
            // }

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
                let time_backtrack_start = Instant::now();
                self.backtrack(backtrack_level);
                let time_backtrack = time_backtrack_start.elapsed();
                self.time_backtrack += time_backtrack;

                let time_learn_start = Instant::now();
                assert!(lemma.len() > 0);
                if lemma.len() == 1 {
                    // Learn a unit clause
                    self.unchecked_enqueue(lemma[0], None);
                    info!(
                        "u {} {} {} {} {} {}",
                        self.decision_level(),
                        self.num_restarts(),
                        self.num_conflicts(),
                        self.num_learnts(),
                        self.num_clauses(),
                        self.num_vars()
                    );
                } else {
                    // Learn a clause
                    let lemma = Rc::new(Clause::new(lemma));
                    self.learnts.push(Rc::clone(&lemma));
                    self.attach_clause(Rc::clone(&lemma));
                    self.unchecked_enqueue(lemma[0], Some(Rc::clone(&lemma)));
                }
                let time_learn = time_learn_start.elapsed();
                self.time_learn += time_learn;

                self.var_decay_activity();
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
                        "r {} {} {} {} {} {}",
                        self.decision_level(),
                        self.num_restarts(),
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
        let ref act = self.activity;
        let ref assign = self.assignment;
        self.order_heap
            .sorted_iter_by(|a, b| match act[a].total_cmp(&act[b]) {
                Ordering::Less => false,
                Ordering::Equal => a.0 < b.0,
                Ordering::Greater => true,
            })
            // .find(|&var| assign.get(var).is_none())
            .find(|&var| assign[var].is_undef())

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
    fn enqueue(&mut self, lit: Lit, reason: Option<Rc<Clause>>) -> bool {
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

    fn unchecked_enqueue(&mut self, lit: Lit, reason: Option<Rc<Clause>>) {
        debug_assert_eq!(self.value(lit), LBool::Undef);

        self.assignment[lit.var()] = LBool::from(!lit.negated());
        self.var_data[lit.var()] = VarData {
            reason,
            level: self.decision_level(),
        };
        self.trail.push(lit);
    }

    fn propagate(&mut self) -> Option<Rc<Clause>> {
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
                    let w = &*i;
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
                        *j = w.clone();
                        j = j.add(1);
                        continue;
                    }

                    // Make sure the false literal is at index 1:
                    if w.clause[0] == false_literal {
                        // println!("swapping {:?} and {:?}", w.clause[0], w.clause[1]);
                        // Rc::get_mut(&mut w.clause).unwrap().lits.swap(0, 1);
                        // FIXME: unsafe!
                        // unsafe {
                        let p = w.clause.lits.as_ptr() as *mut Lit;
                        ptr::swap(p, p.add(1));
                        // *p = *p.add(1);
                        // *p.add(1) = false_literal;
                        // }
                    }
                    debug_assert_eq!(w.clause[1], false_literal, "w.clause[1] must be false_literal");

                    // If the 0th literal is `true`, then the clause is already satisfied
                    // TODO: let first = w.clause[0] & w.clause[1] ^ false_literal;
                    let first = w.clause[0];
                    let w = Watcher {
                        // clause: w.clause,
                        clause: Rc::clone(&w.clause),
                        blocker: first,
                    };
                    if self.value(first) == LBool::True {
                        // self.watchlist.insert(false_literal, w);
                        *j = w;
                        j = j.add(1);
                        continue;
                    }

                    // Find the non-falsified literal:
                    for i in 2..w.clause.lits.len() {
                        let other = w.clause[i];
                        if self.value(other) != LBool::False {
                            // Rc::get_mut(&mut w.clause).unwrap().lits.swap(1, i);
                            // FIXME: unsafe!
                            // unsafe {
                            let p = w.clause.lits.as_ptr() as *mut Lit;
                            ptr::swap(p.add(1), p.add(i));
                            // *p.add(1) = other;
                            // *p.add(i) = false_literal;
                            // }
                            self.watchlist.insert(other, w);
                            continue 'watches;
                        }
                    }

                    match self.value(first) {
                        LBool::Undef => {
                            // unit
                            // debug!("unit {:?} with reason {:?}", first, w.clause);
                            self.unchecked_enqueue(first, Some(Rc::clone(&w.clause)));
                            // self.watchlist.insert(false_literal, w);
                            *j = w;
                            j = j.add(1);
                        }
                        LBool::False => {
                            // conflict
                            // debug!("conflict {:?}", w.clause);
                            conflict = Some(Rc::clone(&w.clause));
                            self.qhead = self.trail.len();
                            // self.watchlist.insert(false_literal, w);
                            *j = w;
                            j = j.add(1);
                            // TODO: copy the remaining watches here
                            while i < end {
                                *j = (*i).clone();
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
    fn analyze(&mut self, conflict: Rc<Clause>) -> (Vec<Lit>, usize) {
        debug!("analyze conflict: {:?}", conflict);

        debug_assert!(self.decision_level() > 0);

        let mut lemma = Vec::new();
        let mut seen = vec![false; self.num_vars()];
        let mut start_index = 0; // 0 for initial conflict, 1 thereafter
        let mut counter: u32 = 0; // number of literals in the conflicting clause on the current decision level
        let mut confl = conflict;
        let mut index = self.trail.len();

        loop {
            // TODO: if conflict.learnt() { bump clause activity for 'conflict' }

            for j in start_index..confl.size() {
                let q = confl[j];
                debug_assert_eq!(self.value(q), LBool::False);

                if !seen[q.var().index()] && self.level(q.var()) > 0 {
                    // TODO: bump var activity for q.var()
                    self.var_bump_activity(q.var());

                    seen[q.var().index()] = true;

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
                if seen[self.trail[index].var().index()] {
                    break;
                }
            }
            let p = self.trail[index];
            seen[p.var().index()] = false; // TODO: why do we need to un-seen p?
            start_index = 1;
            counter -= 1;
            if counter <= 0 {
                // Prepend the asserting literal:
                lemma.insert(0, !p);
                break;
            }
            confl = self.reason(p.var()).unwrap();
            debug_assert_eq!(p, confl[0]);
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
    use std::path::Path;

    use crate::lbool::LBool;
    use crate::lit::Lit;
    use crate::solver::Solver;

    #[test]
    fn test_it_works() {
        let path = "data/coloring.cnf";
        let mut solver = Solver::from_file(Path::new(path));
        let res = solver.solve();
        println!("Solver returned: {:?}", res);
    }

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
