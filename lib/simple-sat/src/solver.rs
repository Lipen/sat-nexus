use std::io::BufRead;
use std::mem;
use std::path::Path;
use std::time::{Duration, Instant};

use tracing::{debug, info};

// use rand::rngs::StdRng;
// use rand::{Rng, SeedableRng};
use crate::assignment::{Assignment, VarData};
use crate::clause::Clause;
use crate::clause_allocator::ClauseAllocator;
use crate::clause_database::ClauseDatabase;
use crate::cref::ClauseRef;
use crate::idx::VarVec;
use crate::lbool::LBool;
use crate::lit::Lit;
use crate::restart::RestartStrategy;
use crate::utils::parse_dimacs_clause;
use crate::utils::read_maybe_gzip;
use crate::var::Var;
use crate::var_order::VarOrder;
use crate::watch::{WatchList, Watcher};

/// CDCL SAT solver.
///
/// **Properties:**
///
/// * `ca`: The clause allocator.
/// * `db`: The clause database.
/// * `watchlist`: A list of clauses that are watched by a variable.
/// * `assignment`: The current assignment of the solver.
/// * `var_order`: The variable order heuristic.
/// * `polarity`: The phase saving heuristic.
/// * `ok`: This is a boolean that indicates whether the solver is in a state where it can continue solving.
/// * `next_var`: The next variable to be assigned.
/// * `decisions`: The number of decisions made by the solver.
/// * `propagations`: The number of times a unit clause was found and propagated.
/// * `conflicts`: The number of conflicts encountered so far.
/// * `restarts`: The number of restarts.
/// * `time_search`: The total time spent in the search function.
/// * `time_propagate`: The time spent in the propagate function.
/// * `time_analyze`: The time spent in conflict analysis.
/// * `time_backtrack`: The time spent backtracking.
/// * `time_decide`: The time spent making a decision.
#[derive(Debug)]
pub struct Solver {
    ca: ClauseAllocator,
    db: ClauseDatabase,
    watchlist: WatchList,
    assignment: Assignment,
    pub var_order: VarOrder,
    polarity: VarVec<bool>, // `pol=true` => negated lit; `false` => positive
    // seen: Vec<bool>,
    pub restart_strategy: RestartStrategy,
    ok: bool,
    next_var: u32,
    // rng: StdRng,
    // Statistics:
    decisions: usize,
    propagations: usize,
    conflicts: usize,
    restarts: usize,
    reduces: usize,
    // Timings:
    pub time_search: Duration,
    pub time_propagate: Duration,
    pub time_analyze: Duration,
    pub time_backtrack: Duration,
    pub time_decide: Duration,
    pub time_restart: Duration,
    pub time_reduce: Duration,
    // For `reduce_db`:
    max_learnts: f64,
    learntsize_factor: f64,
    learntsize_inc: f64,
    learntsize_adjust_cnt: u64,
    learntsize_adjust_confl: f64,
    learntsize_adjust_start: f64,
    learntsize_adjust_inc: f64,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            ca: ClauseAllocator::new(),
            db: ClauseDatabase::new(),
            watchlist: WatchList::new(),
            assignment: Assignment::new(),
            var_order: VarOrder::new(),
            polarity: VarVec::new(),
            // seen: Vec::new(),
            restart_strategy: RestartStrategy::new(),
            ok: true,
            next_var: 0,
            // rng: StdRng::seed_from_u64(42),
            decisions: 0,
            propagations: 0,
            conflicts: 0,
            restarts: 0,
            reduces: 0,
            time_search: Duration::new(0, 0),
            time_propagate: Duration::new(0, 0),
            time_analyze: Duration::new(0, 0),
            time_backtrack: Duration::new(0, 0),
            time_decide: Duration::new(0, 0),
            time_restart: Duration::new(0, 0),
            time_reduce: Duration::new(0, 0),
            max_learnts: f64::MAX,
            learntsize_factor: 0.3,
            learntsize_inc: 1.1,
            learntsize_adjust_cnt: 0,
            learntsize_adjust_confl: 0.0,
            learntsize_adjust_start: 100.0,
            learntsize_adjust_inc: 1.5,
        }
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver {
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

    /// Number of variables.
    pub fn num_vars(&self) -> usize {
        self.next_var as _
    }
    /// Number of original clauses.
    pub fn num_clauses(&self) -> usize {
        self.db.num_clauses()
    }
    /// Number of learnt clauses.
    pub fn num_learnts(&self) -> usize {
        self.db.num_learnts()
    }
    /// Number of decisions.
    pub fn num_decisions(&self) -> usize {
        self.decisions
    }
    /// Number of propagations.
    pub fn num_propagations(&self) -> usize {
        self.propagations
    }
    /// Number of conflicts.
    pub fn num_conflicts(&self) -> usize {
        self.conflicts
    }
    /// Number of restarts.
    pub fn num_restarts(&self) -> usize {
        self.restarts
    }
    /// Number of clause database reductions.
    pub fn num_reduces(&self) -> usize {
        self.reduces
    }

    /// Allocate a new variable.
    pub fn new_var(&mut self) -> Var {
        let var = Var(self.next_var);
        self.next_var += 1;

        // Watch
        self.watchlist.init(var);

        // Assignment
        self.assignment.assignment.push(LBool::Undef);

        // Reason/level
        self.assignment.var_data.push(VarData { reason: None, level: 0 });

        // Polarity
        self.polarity.push(true); // default phase is "negated=true"

        // Seen
        // self.seen.push(false);

        // VSIDS
        self.var_order.push_zero_activity();
        self.var_order.insert_var_order(var);

        // TODO: decision

        // println!("Solver::new_var -> {:?}", v);
        var
    }

    /// Value of the variable.
    pub fn value_var(&self, var: Var) -> LBool {
        self.assignment.value_var(var)
    }
    /// Value of the literal.
    pub fn value(&self, lit: Lit) -> LBool {
        self.assignment.value(lit)
    }

    /// The reason clause for `var`.
    pub fn reason(&self, var: Var) -> Option<ClauseRef> {
        self.assignment.reason(var)
    }
    /// The decision level on which `var` was assigned.
    pub fn level(&self, var: Var) -> usize {
        self.assignment.level(var)
    }

    /// The current decision level.
    pub fn decision_level(&self) -> usize {
        self.assignment.decision_level()
    }

    pub fn clause(&self, cref: ClauseRef) -> &Clause {
        self.ca.clause(cref)
    }
    pub fn clause_mut(&mut self, cref: ClauseRef) -> &mut Clause {
        self.ca.clause_mut(cref)
    }

    pub fn add_clause(&mut self, lits: &[Lit]) -> bool {
        // assert_eq!(self.decision_level(), 0);
        assert!(!lits.is_empty());

        // If the solver is already in UNSAT state, we do not need to add new clause.
        if !self.ok {
            return false;
        }

        // FIXME: actually, this (backtracking to 0th level) has to be done at the end of `solve`,
        //        but currently it is not possible, since we are not saving the model.
        //  Thus, in order to support incremental solving, we backtrack here.
        //  Hopefully, this is going to improve once we implement the model saving.
        if self.decision_level() > 0 {
            self.backtrack(0);
        }

        // Note: We assume that the clause does not have duplicates or tautologies.

        // Auto-create missing variables.
        let max_var = lits.iter().map(|&lit| lit.var().0 + 1).max().unwrap() as _; // 1-based
        assert!(max_var > 0);
        for _ in (self.num_vars() + 1)..=max_var {
            self.new_var();
        }

        // TODO: handle unit clauses (better)

        if lits.len() >= 2 {
            let cref = self.db.add_clause(lits, false, &mut self.ca);
            self.attach_clause(cref);
        } else {
            assert_eq!(lits.len(), 1);
            assert_eq!(self.decision_level(), 0);
            if !self.assignment.enqueue(lits[0], None) {
                // Conflict on 0th level => UNSAT
                self.ok = false;
            }
        }
        self.ok
    }

    fn attach_clause(&mut self, cref: ClauseRef) {
        let clause = self.ca.clause(cref);
        debug_assert!(clause.len() >= 2, "Clause must have at least 2 literals");
        let a = clause[0];
        let b = clause[1];
        self.watchlist.insert(a, Watcher { cref, blocker: b });
        self.watchlist.insert(b, Watcher { cref, blocker: a });
    }

    fn report(&self, stage: &str) {
        info!(
            "{} lvl={} rst={} red={} dec={} prp={} cfl={} lrn={} cls={} vrs={}",
            stage,
            self.decision_level(),
            self.num_restarts(),
            self.num_reduces(),
            self.num_decisions(),
            self.num_propagations(),
            self.num_conflicts(),
            self.num_learnts(),
            self.num_clauses(),
            self.num_vars()
        );
    }

    pub fn solve(&mut self) -> bool {
        // If the solver is already in UNSAT state, return early.
        if !self.ok {
            return false;
        }

        // Make sure to start from the 0th level:
        if self.decision_level() > 0 {
            self.backtrack(0);
        }

        self.max_learnts = self.num_clauses() as f64 * self.learntsize_factor;
        self.learntsize_adjust_confl = self.learntsize_adjust_start;
        self.learntsize_adjust_cnt = self.learntsize_adjust_confl as _;

        if self.restart_strategy.is_luby {
            info!("Using Luby restarts");
        } else {
            info!("Using exponential restarts");
        }

        let mut status = None;
        let mut current_restarts = 0;
        while status.is_none() {
            let num_confl = self.restart_strategy.num_confl(current_restarts);
            let time_search_start = Instant::now();
            status = self.search(num_confl);
            current_restarts += 1;
            let time_search = time_search_start.elapsed();
            self.time_search += time_search;
            debug!("Search #{} done in {:?}", current_restarts, time_search);
        }

        status.unwrap()
    }

    /// The function `search` is the main CDCL loop.
    /// It calls `propagate` to propagate the current assignment, and if a conflict
    /// is found, it calls `analyze` to analyze the conflict and learn a new clause,
    /// and then it calls `backtrack` to backtrack to a lower decision level.
    /// If no conflict is found, it calls `pick_branching_variable` to pick a new variable
    /// to branch on, and then it calls `new_decision_level` to create a new decision level,
    /// and enqueues the new decision via `unchecked_enqueue`.
    ///
    /// **Arguments:**
    ///
    /// * `num_confl`: the number of conflicts before a restart.
    ///
    /// **Returns:**
    ///
    /// - [`Some(true)`][Some] if the formula is satisfiable (no more unassigned variables),
    /// - [`Some(false)`][Some] if it is unsatisfiable (found a conflict on the ground level),
    /// - [`None`] if it is unknown yet (for example, a restart occurred).
    fn search(&mut self, num_confl: usize) -> Option<bool> {
        assert!(self.ok);
        assert_eq!(self.decision_level(), 0);

        let confl_limit = if num_confl > 0 { self.conflicts + num_confl } else { usize::MAX };

        // CDCL loop
        loop {
            // Propagate, analyze, backtrack:
            //  - Returns `true` if everything OK so far
            //  - Returns `false` if conflict on root level was found (UNSAT)
            if !self.propagate_analyze_backtrack() {
                info!("UNSAT");
                return Some(false);
            }

            // Restart:
            if self.conflicts >= confl_limit {
                self.restart();
                return None;
            }

            // Reduce DB:
            let learnts_limit = self.max_learnts + self.assignment.trail.len() as f64;
            if self.num_learnts() as f64 >= learnts_limit {
                self.reduce_db();
            }

            // Make a decision:
            //  - Returns `true` if successfully made a decision.
            //  - Returns `false` if no decision can be made (SAT).
            if !self.decide() {
                info!("SAT");
                return Some(true);
            }
        }
    }

    /// Propagate and then if there is a conflict, analyze it, backtrack, and add the learnt clause.
    ///
    /// **Returns:**
    ///
    /// - `false`, if a conflict on root level was found (UNSAT),
    /// - `true`, otherwise.
    fn propagate_analyze_backtrack(&mut self) -> bool {
        while let Some(conflict) = self.propagate() {
            self.conflicts += 1;

            if self.decision_level() == 0 {
                // conflict on root level => UNSAT
                return false;
            }

            // Analyze the conflict:
            let (lemma, backtrack_level) = self.analyze(conflict);
            debug!("Learnt {:?}", lemma);

            // Backjump:
            self.backtrack(backtrack_level);

            // Add the learnt clause:
            assert!(lemma.len() > 0);
            if lemma.len() == 1 {
                // Learn a unit clause
                debug_assert_eq!(self.decision_level(), 0);
                self.assignment.unchecked_enqueue(lemma[0], None);
                self.report("unit");
            } else {
                // Learn a clause
                let asserting_literal = lemma[0];
                let cref = self.db.add_clause(&lemma, true, &mut self.ca);
                self.attach_clause(cref);
                self.db.cla_bump_activity(cref, &mut self.ca);
                self.assignment.enqueue(asserting_literal, Some(cref));
            }

            self.var_order.var_decay_activity();
            self.db.cla_decay_activity();
            self.update_reduce_db();
        }
        true
    }

    fn propagate(&mut self) -> Option<ClauseRef> {
        let time_propagate_start = Instant::now();

        let mut conflict = None;

        #[inline]
        fn ptr_diff<T>(a: *const T, b: *const T) -> usize {
            ((b as usize) - (a as usize)) / mem::size_of::<T>()
        }

        while let Some(p) = self.assignment.dequeue() {
            debug_assert_eq!(self.level(p.var()), self.decision_level());
            // debug!("Propagating {:?}", p);
            self.propagations += 1;
            let false_literal = !p;

            unsafe {
                let watchers = self.watchlist.lookup(false_literal);
                let ws = watchers.as_mut_ptr();

                let begin = ws;
                let end = begin.add(watchers.len());

                let mut i = begin;
                let mut j = begin;

                'watches: while i < end {
                    let Watcher { cref, blocker } = *i;
                    i = i.add(1);

                    let clause = self.ca.clause_mut(cref);

                    // Skip the deleted clause:
                    if clause.is_deleted() {
                        continue;
                    }

                    // Try to avoid inspecting the clause:
                    if self.assignment.value(blocker) == LBool::True {
                        *j = Watcher { cref, blocker };
                        j = j.add(1);
                        continue;
                    }

                    // Make sure the false literal is at index 1:
                    if clause[0] == false_literal {
                        clause[0] = clause[1];
                        clause[1] = false_literal;
                    }
                    debug_assert_eq!(clause[1], false_literal, "clause[1] must be false_literal");

                    // If the first literal is `true`, then the clause is already satisfied
                    // TODO: let first = w.clause[0] & w.clause[1] ^ false_literal;
                    let first = clause[0];
                    if first != blocker && self.assignment.value(first) == LBool::True {
                        *j = Watcher { cref, blocker: first };
                        j = j.add(1);
                        continue;
                    }

                    // Find the non-falsified literal:
                    for i in 2..clause.len() {
                        let other = clause[i];
                        if self.assignment.value(other) != LBool::False {
                            clause[1] = other;
                            clause[i] = false_literal;
                            self.watchlist.insert(other, Watcher { cref, blocker: first });
                            continue 'watches;
                        }
                    }

                    *j = Watcher { cref, blocker: first };
                    j = j.add(1);
                    match self.assignment.value(first) {
                        LBool::Undef => {
                            // unit
                            debug!("Propagated unit {:?} with reason {:?} = {:?}", first, cref, self.clause(cref));
                            self.assignment.unchecked_enqueue(first, Some(cref));
                        }
                        LBool::False => {
                            // conflict
                            debug!("Found conflict: {:?} = {:?}", cref, self.clause(cref));
                            conflict = Some(cref);
                            self.assignment.qhead = self.assignment.trail.len();
                            // Copy the remaining watches:
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

        self.time_propagate += time_propagate_start.elapsed();
        conflict
    }

    /// Returns learnt clause and backtrack level.
    fn analyze(&mut self, conflict: ClauseRef) -> (Vec<Lit>, usize) {
        debug!(
            "Analyze conflict @{}: {:?} = {:?}",
            self.decision_level(),
            conflict,
            self.clause(conflict)
        );
        debug_assert!(self.decision_level() > 0);

        let time_analyze_start = Instant::now();

        let mut lemma = Vec::new();
        let mut seen = VarVec::from(vec![false; self.num_vars()]);
        let mut active: u32 = 0; // number of literals in the conflicting clause on the current decision level
        let mut reason = conflict;
        let mut index = self.assignment.trail.len();

        loop {
            // Bump `reason` clause activity:
            self.db.cla_bump_activity(reason, &mut self.ca);

            let clause = self.ca.clause(reason);
            let start_index = if reason == conflict { 0 } else { 1 };
            for &q in &clause[start_index..] {
                debug_assert_eq!(self.value(q), LBool::False);

                if !seen[q.var()] && self.level(q.var()) > 0 {
                    seen[q.var()] = true;

                    // Bump `q` variable activity:
                    self.var_order.var_bump_activity(q.var());

                    if self.level(q.var()) < self.decision_level() {
                        lemma.push(q);
                    } else {
                        debug_assert_eq!(self.level(q.var()), self.decision_level());
                        active += 1;
                    }
                }
            }

            // Select next clause (`reason`) to look at:
            // let &p = self.assignment.trail.iter().rfind(|p| seen[p.var()]).unwrap();
            loop {
                index -= 1;
                if seen[self.assignment.trail[index].var()] {
                    break;
                }
            }
            let p = self.assignment.trail[index];
            debug_assert_eq!(self.level(p.var()), self.decision_level());
            seen[p.var()] = false;
            active -= 1;
            if active == 0 {
                // Prepend the asserting literal:
                lemma.insert(0, !p);
                break;
            }
            reason = self.reason(p.var()).unwrap();
            debug_assert_eq!(self.clause(reason)[0], p);
        }

        // Save learnt literals for later usage:
        let analyze_to_clear = lemma.clone();

        // Minimize the learnt clause:
        // Note: currently, only "local" minimization (i.e. not "recursive") is implemented.
        lemma.retain(|&lit| !self.lit_redundant_basic(lit, &seen));

        // Clear the `seen` vector:
        for lit in analyze_to_clear {
            seen[lit.var()] = false;
        }
        debug_assert!(seen.iter().all(|&x| !x));

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

        self.time_analyze += time_analyze_start.elapsed();
        (lemma, bt_level)
    }

    fn lit_redundant_basic(&self, lit: Lit, seen: &VarVec<bool>) -> bool {
        match self.reason(lit.var()) {
            None => false,
            Some(cref) => {
                let clause = self.ca.clause(cref);
                debug_assert_eq!(clause[0], !lit);
                for &x in &clause[1..] {
                    let v = x.var();
                    if !seen[v] && self.level(v) > 0 {
                        return false;
                    }
                }
                true
            }
        }
    }

    fn backtrack(&mut self, level: usize) {
        debug!("backtrack from {} to {}", self.decision_level(), level);

        let time_backtrack_start = Instant::now();

        if self.decision_level() > level {
            for i in (self.assignment.trail_lim[level]..self.assignment.trail.len()).rev() {
                let var = self.assignment.trail[i].var();
                // Phase saving:
                self.polarity[var] = !self.assignment.value_var(var).bool().expect("must be assigned");
                // Unassign the variable:
                self.assignment.unassign(var);
                // Put the variable into the order heap:
                self.var_order.insert_var_order(var);
            }
            self.assignment.qhead = self.assignment.trail_lim[level];
            self.assignment.trail.truncate(self.assignment.trail_lim[level]);
            self.assignment.trail_lim.truncate(level);
        }

        self.time_backtrack += time_backtrack_start.elapsed();
    }

    /// If there is a variable that is unassigned, then pick it and assign.
    ///
    /// **Returns:**
    ///
    /// - `true`, if successfully made a decision,
    /// - `false`, if there are no unassigned variables (SAT).
    fn decide(&mut self) -> bool {
        let time_decide_start = Instant::now();
        let ok = if let Some(var) = self.pick_branching_variable() {
            let decision = self.pick_phase(var);

            debug!(
                "Made a decision = {:?} = {}{:?}",
                decision,
                if decision.negated() { "-" } else { "+" },
                decision.var()
            );

            self.decisions += 1;
            self.assignment.new_decision_level();
            self.assignment.unchecked_enqueue(decision, None);

            true
        } else {
            false
        };
        self.time_decide += time_decide_start.elapsed();
        ok
    }

    fn pick_branching_variable(&mut self) -> Option<Var> {
        self.var_order.pick_branching_variable(&self.assignment)
    }

    fn pick_phase(&mut self, var: Var) -> Lit {
        // let sign = self.rng.gen(); // random phase
        // let sign = false; // always positive phase
        // let sign = true; // always negative phase
        let sign = self.polarity[var];

        Lit::new(var, sign)
    }

    fn restart(&mut self) {
        let time_restart_start = Instant::now();
        self.restarts += 1;
        self.report("restart");
        self.backtrack(0);
        self.time_restart += time_restart_start.elapsed();
    }

    fn update_reduce_db(&mut self) {
        self.learntsize_adjust_cnt -= 1;
        if self.learntsize_adjust_cnt == 0 {
            self.learntsize_adjust_confl *= self.learntsize_adjust_inc;
            self.learntsize_adjust_cnt = self.learntsize_adjust_confl as _;
            self.max_learnts *= self.learntsize_inc;
            debug!(
                "New max_learnts = {}, learntsize_adjust_cnt = {}",
                self.max_learnts as u64, self.learntsize_adjust_cnt
            );
        }
    }

    fn reduce_db(&mut self) {
        let time_reduce_start = Instant::now();
        self.reduces += 1;
        self.report("reduce");
        self.db.reduce(&self.assignment, &mut self.ca);
        self.time_reduce += time_reduce_start.elapsed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_log::test;

    #[test]
    fn test_correctness() {
        let mut solver = Solver::new();

        let tie = Lit::new(solver.new_var(), false);
        let shirt = Lit::new(solver.new_var(), false);
        info!("TIE = {:?}, SHIRT = {:?}", tie, shirt);
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
