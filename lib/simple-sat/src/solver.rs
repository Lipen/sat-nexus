use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::mem;
use std::path::Path;
use std::time::{Duration, Instant};

use itertools::{zip_eq, Itertools};
use serde_with::SerializeDisplay;
use tracing::{debug, info, trace, warn};

// use rand::rngs::StdRng;
// use rand::{Rng, SeedableRng};
use crate::assignment::{Assignment, VarData};
use crate::clause::Clause;
use crate::clause_allocator::ClauseAllocator;
use crate::clause_database::ClauseDatabase;
use crate::cref::ClauseRef;
use crate::idx::VarVec;
use crate::lbool::LBool;
use crate::learning::{LearningGuard, LearningStrategy};
use crate::lit::Lit;
use crate::options::Options;
use crate::options::DEFAULT_OPTIONS;
use crate::restart::RestartStrategy;
use crate::trie::Trie;
use crate::utils::parse_dimacs;
use crate::utils::DisplaySlice;
use crate::var::Var;
use crate::var_order::VarOrder;
use crate::watch::{WatchList, Watcher};

#[derive(Debug, Copy, Clone, Eq, PartialEq, SerializeDisplay)]
pub enum SolveResult {
    Sat,
    Unsat,
    Unknown,
}

impl Display for SolveResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SolveResult::Sat => "SAT",
                SolveResult::Unsat => "UNSAT",
                SolveResult::Unknown => "UNKNOWN",
            }
        )
    }
}

#[derive(Debug)]
enum SearchResult {
    Sat,
    Unsat,
    Restart,
    AssumptionsConflict(Vec<Lit>),
}

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
    options: Options,
    ca: ClauseAllocator,
    db: ClauseDatabase,
    watchlist: WatchList,
    assignment: Assignment,
    var_order: VarOrder,
    polarity: VarVec<bool>, // `pol=true` => negated lit; `false` => positive
    // seen: Vec<bool>,
    pub restart_strategy: RestartStrategy,
    pub learning_guard: LearningGuard,
    ok: bool,
    next_var: u32,
    // rng: StdRng,
    // Statistics:
    decisions: usize,
    propagations: usize,
    conflicts: usize,
    restarts: usize,
    simplifies: usize,
    reduces: usize,
    // Timings:
    pub time_search: Duration,
    pub time_propagate: Duration,
    pub time_analyze: Duration,
    pub time_backtrack: Duration,
    pub time_decide: Duration,
    pub time_restart: Duration,
    pub time_simplify: Duration,
    pub time_reduce: Duration,
}

impl Solver {
    pub fn new(options: Options) -> Self {
        let restart_strategy = RestartStrategy {
            is_luby: options.is_luby,
            restart_init: options.restart_init,
            restart_inc: options.restart_inc,
        };
        let learning_strategy = LearningStrategy {
            min_learnts_limit: options.min_learnts_limit,
            learntsize_factor: options.learntsize_factor,
            learntsize_inc: options.learntsize_inc,
            learntsize_adjust_start: options.learntsize_adjust_start,
            learntsize_adjust_inc: options.learntsize_adjust_inc,
        };
        let learning_guard = LearningGuard::new(learning_strategy);
        Self {
            options,
            ca: ClauseAllocator::new(),
            db: ClauseDatabase::new(),
            watchlist: WatchList::new(),
            assignment: Assignment::new(),
            var_order: VarOrder::new(),
            polarity: VarVec::new(),
            // seen: Vec::new(),
            restart_strategy,
            learning_guard,
            ok: true,
            next_var: 0,
            // rng: StdRng::seed_from_u64(42),
            decisions: 0,
            propagations: 0,
            conflicts: 0,
            restarts: 0,
            simplifies: 0,
            reduces: 0,
            time_search: Duration::new(0, 0),
            time_propagate: Duration::new(0, 0),
            time_analyze: Duration::new(0, 0),
            time_backtrack: Duration::new(0, 0),
            time_decide: Duration::new(0, 0),
            time_restart: Duration::new(0, 0),
            time_simplify: Duration::new(0, 0),
            time_reduce: Duration::new(0, 0),
        }
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self::new(DEFAULT_OPTIONS)
    }
}

impl Display for Solver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Solver")
    }
}

impl Solver {
    pub fn init_from_file<P>(&mut self, path: P)
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        debug!("Initializing solver from '{}'", path.display());
        for clause in parse_dimacs(path) {
            self.add_clause(&clause);
        }
    }

    /// Number of variables.
    pub fn num_vars(&self) -> usize {
        self.next_var as usize
    }
    /// Number of free variables.
    pub fn num_free_vars(&self) -> usize {
        let num_ground_assignments = if self.assignment.trail_lim.is_empty() {
            self.assignment.trail.len()
        } else {
            self.assignment.trail_lim[0]
        };
        self.var_order.num_dec_vars - num_ground_assignments
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
    /// Number of clause database simplifies.
    pub fn num_simplifies(&self) -> usize {
        self.simplifies
    }
    /// Number of clause database reductions.
    pub fn num_reduces(&self) -> usize {
        self.reduces
    }

    /// Reset the solver state.
    pub fn reset(&mut self) {
        let options = self.options.clone();
        *self = Self::new(options);
    }

    /// Allocate a new variable.
    pub fn new_var(&mut self) -> Var {
        let var = Var::new(self.next_var);
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
        self.var_order.init_var(var);
        // self.var_order.push_zero_activity();
        // self.var_order.insert_var_order(var);

        // TODO: decision

        // println!("Solver::new_var -> {:?}", v);
        var
    }

    /// Allocate a new variable and return it as positive literal.
    pub fn new_lit(&mut self) -> Lit {
        Lit::new(self.new_var(), false)
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

    pub fn clauses_iter(&self) -> impl Iterator<Item = &Clause> + '_ {
        self.db.clauses().iter().map(|&cref| self.ca.clause(cref))
    }
    pub fn learnts_iter(&self) -> impl Iterator<Item = &Clause> + '_ {
        self.db.learnts().iter().map(|&cref| self.ca.clause(cref))
    }

    pub fn add_clause(&mut self, lits: &[Lit]) -> bool {
        // assert_eq!(self.decision_level(), 0);

        if lits.is_empty() {
            warn!("Empty clause");
            self.ok = false;
            return false;
        }

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
        let max_var = lits.iter().map(|&lit| lit.var()).max().unwrap();
        for _ in (self.num_vars() + 1)..=max_var.to_external() as _ {
            self.new_var();
        }

        // TODO: handle unit clauses (better)

        if lits.len() >= 2 {
            let cref = self.db.new_clause(lits, false, &mut self.ca);
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

    pub fn add_learnt(&mut self, lits: &[Lit]) -> bool {
        assert_eq!(self.decision_level(), 0);

        if lits.is_empty() {
            warn!("Empty learnt clause");
            self.ok = false;
        }

        if !self.ok {
            // Already UNSAT, no need to add learnts.
            return false;
        }

        if lits.len() == 1 {
            // Learn a unit clause:
            let res = self.assignment.enqueue(lits[0], None);
            assert!(res);
            // FIXME: handle (ignore, in fact) the bool returned from 'enqueue'

            // Propagate the assigned unit:
            if let Some(conflict) = self.propagate() {
                debug!("Conflict during propagation of learnt unit: {}", self.clause(conflict));
                self.ok = false;
            }
        } else {
            // Learn a clause:
            let cref = self.db.new_clause(lits, true, &mut self.ca);
            self.attach_clause(cref);
            self.db.cla_bump_activity(cref, &mut self.ca);
        }

        // Post-simplify:
        self.simplify();

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
            "{} lvl={} rst={} red={} dec={} prp={} cfl={} lrn={} cls={} vrs={} free={}",
            stage,
            self.decision_level(),
            self.num_restarts(),
            self.num_reduces(),
            self.num_decisions(),
            self.num_propagations(),
            self.num_conflicts(),
            self.num_learnts(),
            self.num_clauses(),
            self.num_vars(),
            self.num_free_vars(),
        );
    }

    pub fn solve(&mut self) -> SolveResult {
        self.solve_under_assumptions(&[])
    }

    pub fn solve_under_assumptions(&mut self, assumptions: &[Lit]) -> SolveResult {
        // If the solver is already in UNSAT state, return early.
        if !self.ok {
            return SolveResult::Unsat;
        }

        // Make sure to start from the 0th level:
        if self.decision_level() > 0 {
            self.backtrack(0);
        }

        // Reset the limits for reduceDB:
        self.learning_guard.reset(self.num_clauses());

        if self.restart_strategy.is_luby {
            debug!("Using Luby restarts");
        } else {
            debug!("Using exponential restarts");
        }

        let mut status = SolveResult::Unknown;
        let mut current_restarts = 0;
        while status == SolveResult::Unknown {
            let num_confl = self.restart_strategy.num_confl(current_restarts);
            let time_search_start = Instant::now();
            match self.search(num_confl, assumptions) {
                SearchResult::Sat => {
                    status = SolveResult::Sat;
                }
                SearchResult::Unsat => {
                    status = SolveResult::Unsat;
                }
                SearchResult::Restart => {
                    // Restart => do nothing here
                    if false {
                        use std::io::Write;
                        let f = std::fs::File::create("learnts.txt").unwrap();
                        let mut f = std::io::LineWriter::new(f);
                        for lemma in self.learnts_iter() {
                            for lit in lemma.lits() {
                                write!(f, "{} ", lit).unwrap();
                            }
                            writeln!(f, "0").unwrap();
                        }
                    }
                }
                SearchResult::AssumptionsConflict(_conflict) => {
                    // TODO: save the `conflict`
                    status = SolveResult::Unsat;
                }
            }
            let time_search = time_search_start.elapsed();
            self.time_search += time_search;
            current_restarts += 1;
            debug!("Search #{} done in {:?}", current_restarts, time_search);
        }
        status
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
    fn search(&mut self, num_confl: usize, assumptions: &[Lit]) -> SearchResult {
        assert!(self.ok);
        assert_eq!(self.decision_level(), 0);

        let confl_limit = if num_confl > 0 { self.conflicts + num_confl } else { usize::MAX };

        // CDCL loop
        loop {
            // Propagate, analyze, backtrack:
            //  - Returns `true` if everything OK so far
            //  - Returns `false` if conflict on root level was found (UNSAT)
            if !self.propagate_analyze_backtrack() {
                debug!("UNSAT");
                return SearchResult::Unsat;
            }

            // Restart:
            if self.conflicts >= confl_limit {
                self.restart();
                return SearchResult::Restart;
            }

            // Simplify DB:
            if self.decision_level() == 0 {
                // TODO: handle returned 'false' value
                self.simplify();
            }

            // Reduce DB:
            let learnts_limit = self.learning_guard.limit(self.assignment.trail.len());
            if self.num_learnts() >= learnts_limit {
                self.reduce_db();
            }

            // Make a decision:
            //  - Returns `true` if successfully made a decision.
            //  - Returns `false` if no decision can be made (SAT).
            match self.decide(assumptions) {
                Ok(Some(decision)) => {
                    self.assignment.new_decision_level();
                    self.assignment.unchecked_enqueue(decision, None);
                }
                Ok(None) => {
                    debug!("SAT");
                    return SearchResult::Sat;
                }
                Err(conflict) => {
                    debug!("UNSAT (assumptions conflict)");
                    return SearchResult::AssumptionsConflict(conflict);
                }
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
            trace!("Learnt {:?}", lemma);

            // Backjump:
            self.backtrack(backtrack_level);

            // Add the learnt clause:
            assert!(!lemma.is_empty());
            if lemma.len() == 1 {
                // Learn a unit clause
                debug_assert_eq!(self.decision_level(), 0);
                self.assignment.unchecked_enqueue(lemma[0], None);
                self.report("unit");
            } else {
                // Learn a clause
                let asserting_literal = lemma[0];
                let cref = self.db.new_clause(lemma, true, &mut self.ca);
                self.attach_clause(cref);
                self.db.cla_bump_activity(cref, &mut self.ca);
                self.assignment.unchecked_enqueue(asserting_literal, Some(cref));
            }

            self.var_order.var_decay_activity();
            self.db.cla_decay_activity();
            self.learning_guard.bump();
        }
        true
    }

    pub fn propagate(&mut self) -> Option<ClauseRef> {
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
                            trace!("Propagated unit {:?} with reason {:?} = {:?}", first, cref, self.clause(cref));
                            self.assignment.unchecked_enqueue(first, Some(cref));
                        }
                        LBool::False => {
                            // conflict
                            trace!("Found conflict: {:?} = {:?}", cref, self.clause(cref));
                            debug_assert!(conflict.is_none());
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
        trace!("Analyze conflict {} at level {}", self.clause(conflict), self.decision_level());
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
        trace!("Backtrack from {} to {}", self.decision_level(), level);

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

    fn analyze_final(&mut self, p: Lit) -> Vec<Lit> {
        assert_eq!(self.value(p), LBool::True);

        let mut conflict = vec![p];

        if self.decision_level() == 0 {
            return conflict;
        }

        let mut seen = VarVec::from(vec![false; self.num_vars()]);
        seen[p.var()] = true;

        for &lit in self.assignment.trail[self.assignment.trail_lim[0]..].iter().rev() {
            let var = lit.var();
            if seen[var] {
                if let Some(reason) = self.reason(var) {
                    let reason = self.clause(reason);
                    assert_eq!(reason[0], lit);
                    for c in &reason[1..] {
                        let v = c.var();
                        if self.level(v) > 0 {
                            seen[v] = true;
                        }
                    }
                } else {
                    assert!(self.level(var) > 0);
                    conflict.push(!lit);
                }
                seen[var] = false;
            }
        }
        seen[p.var()] = false;

        conflict
    }

    #[allow(unused)]
    fn analyze_full(&mut self, conflict: ClauseRef) -> Vec<Lit> {
        trace!(
            "Analyze (FULL) conflict {} at level {}",
            self.clause(conflict),
            self.decision_level()
        );
        assert!(self.decision_level() > 0);

        let mut lemma = Vec::new();
        let mut seen = VarVec::from(vec![false; self.num_vars()]);

        for p in self.clause(conflict).iter() {
            seen[p.var()] = true;
        }

        for &lit in self.assignment.trail[self.assignment.trail_lim[0]..].iter().rev() {
            let var = lit.var();
            if seen[var] {
                if let Some(reason) = self.reason(var) {
                    let reason = self.clause(reason);
                    assert_eq!(reason[0], lit);
                    for c in &reason[1..] {
                        let v = c.var();
                        if self.level(v) > 0 {
                            seen[v] = true;
                        }
                    }
                } else {
                    assert!(self.level(var) > 0);
                    lemma.push(!lit);
                }
                seen[var] = false;
            }
        }

        lemma
    }

    /// If there is a variable that is unassigned, then pick it and assign.
    ///
    /// **Returns:**
    ///
    /// - `true`, if successfully made a decision,
    /// - `false`, if there are no unassigned variables (SAT).
    fn decide(&mut self, assumptions: &[Lit]) -> Result<Option<Lit>, Vec<Lit>> {
        // Handle assumptions:
        while let Some(&p) = assumptions.get(self.decision_level()) {
            match self.value(p) {
                LBool::True => {
                    // Dummy decision level:
                    // TODO: do we need this dummy level at all?
                    self.assignment.new_decision_level();
                }
                LBool::False => {
                    let conflict = self.analyze_final(!p);
                    return Err(conflict);
                }
                LBool::Undef => {
                    return Ok(Some(p));
                }
            }
        }

        let time_decide_start = Instant::now();
        let decision = if let Some(var) = self.pick_branching_variable() {
            let decision = self.pick_phase(var);
            debug!(
                "Made a decision = {:?} = {}{:?}",
                decision,
                if decision.negated() { "-" } else { "+" },
                decision.var()
            );
            self.decisions += 1;
            Some(decision)
        } else {
            None
        };
        self.time_decide += time_decide_start.elapsed();
        Ok(decision)
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

    pub fn simplify(&mut self) {
        // TODO: return bool
        let time_simplify_start = Instant::now();
        self.simplifies += 1;
        assert_eq!(self.decision_level(), 0);
        self.db.simplify(&self.assignment, &mut self.ca);
        self.time_simplify += time_simplify_start.elapsed();
    }

    fn reduce_db(&mut self) {
        let time_reduce_start = Instant::now();
        self.reduces += 1;
        self.report("reduce");
        self.db.reduce(&self.assignment, &mut self.ca);
        self.time_reduce += time_reduce_start.elapsed();
    }
}

impl Solver {
    pub fn propcheck_num_propagated(&mut self, assumptions: &[Lit]) -> (bool, u64) {
        let mut num_propagated = 0;
        let res = self.propcheck(assumptions, None, Some(&mut num_propagated));
        (res, num_propagated)
    }

    pub fn propcheck_save_propagated(&mut self, assumptions: &[Lit]) -> (bool, Vec<Lit>) {
        let mut propagated = Vec::new();
        let res = self.propcheck(assumptions, Some(&mut propagated), None);
        (res, propagated)
    }

    pub fn propcheck(&mut self, assumptions: &[Lit], out_propagated: Option<&mut Vec<Lit>>, out_num_propagated: Option<&mut u64>) -> bool {
        debug!("propcheck(assumptions = {})", DisplaySlice(assumptions));

        // First, propagate everything that needs to be propagated:
        if let Some(_conflict) = self.propagate() {
            warn!("Conflict during pre-propagation");
            self.ok = false;
        }

        if !self.ok {
            return false;
        }

        // Save the original decision level in order to backtrack to it later:
        let level = self.decision_level();
        let mut conflicting_assignment = false;
        let mut conflict: Option<ClauseRef> = None;

        for &p in assumptions.iter() {
            match self.value(p) {
                LBool::True => {
                    // do nothing
                }
                LBool::False => {
                    // conflict with assumption
                    // info!(
                    //     "Conflict during assignment: value(p = {}) = {:?} with reason = {}",
                    //     p,
                    //     self.value(p),
                    //     self.clause(self.reason(p.var()).unwrap())
                    // );
                    conflicting_assignment = true;
                    break;
                }
                LBool::Undef => {
                    self.assignment.new_decision_level();
                    self.assignment.unchecked_enqueue(p, None);
                    if let Some(c) = self.propagate() {
                        // conflict during propagation
                        // info!("Conflict during propagation: {}", self.clause(c));
                        conflict = Some(c);
                        break;
                    }
                }
            }
        }

        if self.decision_level() > level {
            if let Some(out_propagated) = out_propagated {
                out_propagated.clear();
                for &lit in &self.assignment.trail[self.assignment.trail_lim[level]..] {
                    out_propagated.push(lit);
                }
                if let Some(conflict) = conflict {
                    let conflict = self.clause(conflict);
                    out_propagated.push(conflict[0]);
                }
            }

            if let Some(out_num_propagated) = out_num_propagated {
                *out_num_propagated = (self.assignment.trail.len() - self.assignment.trail_lim[level]) as u64;
                if conflict.is_some() {
                    *out_num_propagated += 1;
                }
            }

            // Backtrack to the original decision level:
            self.backtrack(level);
        }

        conflict.is_none() && !conflicting_assignment
    }

    pub fn propcheck_all(&mut self, variables: &[Var]) -> u64 {
        debug!("propcheck_all(variables = {})", DisplaySlice(variables));

        assert!(variables.len() < 30);

        // TODO: backtrack(0) manually instead of asserting.
        assert_eq!(self.decision_level(), 0);

        let mut cube = vec![false; variables.len()];
        let mut total_checked = 0u64; // number of 'propcheck' calls
        let mut total_count = 0u64; // number of valid cubes

        loop {
            trace!("cube = {}", DisplaySlice(&cube));
            let assumptions = zip_eq(variables, &cube).map(|(&v, &s)| Lit::new(v, s)).collect_vec();
            let res = self.propcheck(&assumptions, None, None);
            total_checked += 1;

            if res {
                trace!("valid assumptions: {}", DisplaySlice(&assumptions));
                total_count += 1;
            } else {
                trace!("invalid assumptions: {}", DisplaySlice(&assumptions));
            }

            // Find the 1-based index of the last 'false' value in 'cube':
            let mut j = variables.len(); // 1-based
            while j > 0 && cube[j - 1] {
                j -= 1;
            }
            if j == 0 {
                break;
            }

            // Increment the 'cube':
            assert!(!cube[j - 1]);
            cube[j - 1] = true;
            for i in j..variables.len() {
                cube[i] = false;
            }
        }

        debug!("Checked {} cubes, {} valid", total_checked, total_count);
        total_count
    }

    pub fn propcheck_all_tree(&mut self, variables: &[Var], limit: u64, add_learnts: bool, out_learnts: &mut Vec<Vec<Lit>>) -> u64 {
        debug!("propcheck_all_tree(variables = {})", DisplaySlice(variables));

        assert!(variables.len() < 30);

        // TODO: backtrack(0) manually instead of asserting.
        assert_eq!(self.decision_level(), 0);

        // Propagate everything that needs to be propagated:
        if let Some(_conflict) = self.propagate() {
            warn!("Conflict during pre-propagation");
            self.ok = false;
        }

        if !self.ok {
            return 0;
        }

        // Trivial case:
        if variables.is_empty() {
            return 0;
        }

        let mut cube = vec![false; variables.len()];
        let mut total_checked = 0u64;
        let mut total_count = 0u64;

        let mut learnts = HashSet::new();

        // Reset the limits for reduceDB:
        // self.learning_guard.reset(self.num_clauses());
        // Note: currently, this is performed outside.

        #[derive(Debug)]
        enum State {
            Descending,
            Ascending,
            Propagating,
        }
        let mut state = State::Descending;

        loop {
            trace!(
                "state = {:?}, cube = {}, level = {}, trail = [{}]",
                state,
                DisplaySlice(&cube),
                self.decision_level(),
                zip_eq(variables, &cube)
                    .take(self.decision_level())
                    .map(|(&v, &s)| Lit::new(v, s))
                    .map(|lit| format!("{}@{}", lit, self.level(lit.var())))
                    .join(", ")
            );
            assert!(self.decision_level() <= variables.len());

            match state {
                State::Descending => {
                    if self.decision_level() == variables.len() {
                        trace!("Found valid cube: {}", DisplaySlice(&cube));
                        total_count += 1;
                        if limit > 0 && total_count > limit {
                            break;
                        }
                        state = State::Ascending;
                    } else {
                        self.assignment.new_decision_level();
                        let v = variables[self.decision_level() - 1];
                        let s = cube[self.decision_level() - 1];
                        let p = Lit::new(v, s);
                        trace!("Trying to assign p = {} at new level {}", p, self.decision_level());
                        match self.value(p) {
                            LBool::True => {
                                trace!("Literal {} already has True value", p);
                                // do nothing
                                // state = State::Descending;
                            }
                            LBool::False => {
                                trace!(
                                    "Propagated different value for p = {} with trail = [{}]",
                                    p,
                                    zip_eq(variables, &cube)
                                        .take(self.decision_level())
                                        .map(|(&v, &s)| Lit::new(v, s))
                                        .map(|lit| format!("{}@{}", lit, self.level(lit.var())))
                                        .join(", ")
                                );
                                state = State::Ascending;
                            }
                            LBool::Undef => {
                                trace!("Enqueueing {}", p);
                                self.assignment.unchecked_enqueue(p, None);
                                state = State::Propagating;
                            }
                        }
                    }
                }

                State::Ascending => {
                    assert!(self.decision_level() > 0);

                    // Find the 1-based index of the last 'false' value in 'cube':
                    let mut j = self.decision_level(); // 1-based
                    while j > 0 && cube[j - 1] {
                        j -= 1;
                    }
                    if j == 0 {
                        break;
                    }

                    // Increment the 'cube':
                    assert!(!cube[j - 1]);
                    cube[j - 1] = true;
                    for i in j..variables.len() {
                        cube[i] = false;
                    }

                    // Backtrack to the level before `j`:
                    self.backtrack(j - 1);

                    // Switch state to descending:
                    state = State::Descending;
                }

                State::Propagating => {
                    total_checked += 1;
                    if let Some(conflict) = self.propagate() {
                        trace!(
                            "Conflict {} for trail = [{}]",
                            self.clause(conflict),
                            zip_eq(variables, &cube)
                                .take(self.decision_level())
                                .map(|(&v, &s)| Lit::new(v, s))
                                .map(|lit| format!("{}@{}", lit, self.level(lit.var())))
                                .join(", ")
                        );

                        if add_learnts {
                            // let lemma = self.analyze_full(conflict);
                            let (lemma, _) = self.analyze(conflict);
                            trace!(
                                "lemma {} for conflict {} with trail = [{}]",
                                DisplaySlice(&lemma),
                                self.clause(conflict),
                                zip_eq(variables, &cube)
                                    .take(self.decision_level())
                                    .map(|(&v, &s)| Lit::new(v, s))
                                    .map(|lit| format!("{}@{}", lit, self.level(lit.var())))
                                    .join(", ")
                            );

                            if learnts.insert(lemma.clone()) {
                                // Add non-unit learnt clause:
                                assert!(!lemma.is_empty());
                                if lemma.len() > 1 {
                                    debug!("Adding learnt {}", DisplaySlice(&lemma));
                                    let cref = self.db.new_clause(lemma, true, &mut self.ca);
                                    self.attach_clause(cref);
                                    self.db.cla_bump_activity(cref, &mut self.ca);
                                }

                                self.var_order.var_decay_activity();
                                self.db.cla_decay_activity();
                                self.learning_guard.bump();
                            } else {
                                trace!("lemma {} already present", DisplaySlice(&lemma));
                            }
                        }

                        state = State::Ascending;
                    } else {
                        trace!(
                            "No conflict for trail = [{}]",
                            zip_eq(variables, &cube)
                                .take(self.decision_level())
                                .map(|(&v, &s)| Lit::new(v, s))
                                .map(|lit| format!("{}@{}", lit, self.level(lit.var())))
                                .join(", ")
                        );
                        state = State::Descending;
                    }
                }
            }
        }

        // Post-backtrack to zero level:
        self.backtrack(0);

        if add_learnts {
            // Add learnt units only:
            for lemma in learnts {
                assert!(!lemma.is_empty());
                if lemma.len() == 1 {
                    assert_eq!(self.decision_level(), 0);
                    debug!("Adding unit {}", lemma[0]);
                    self.assignment.enqueue(lemma[0], None);
                }

                out_learnts.push(lemma);
            }

            self.var_order.var_decay_activity();
            self.db.cla_decay_activity();
            self.learning_guard.bump();

            // TODO: re-check all hard tasks - they might be "easy" after adding the learnt clauses.
        }

        // Post-propagate:
        if let Some(_conflict) = self.propagate() {
            warn!("Conflict during post-propagation");
            self.ok = false;
            return 0;
        }

        if add_learnts {
            // Simplify DB:
            self.simplify();

            // Reduce DB:
            let learnts_limit = self.learning_guard.limit(self.assignment.trail.len());
            if self.num_learnts() >= learnts_limit {
                // self.reduce_db();
                debug!("Reducing DB");
                self.db.reduce(&self.assignment, &mut self.ca);
            }
        }

        debug!("Checked {} cubes, {} valid", total_checked, total_count);
        total_count
    }

    pub fn propcheck_all_trie(&mut self, variables: &[Var], trie: &Trie, valid: &mut Vec<Vec<Lit>>) -> u64 {
        debug!("propcheck_all_trie(variables = {})", DisplaySlice(variables));

        // TODO: backtrack(0) manually instead of asserting.
        assert_eq!(self.decision_level(), 0);

        // Propagate everything that needs to be propagated:
        if let Some(_conflict) = self.propagate() {
            warn!("Conflict during pre-propagation");
            self.ok = false;
        }

        if !self.ok {
            return 0;
        }

        // Trivial case:
        if trie.is_empty() {
            return 0;
        }

        let mut cube = vec![false; variables.len()];

        let mut total_checked = 0u64;
        let mut total_count = 0u64;

        #[derive(Debug)]
        enum State {
            Descending,
            Ascending,
            Propagating,
        }
        let mut state = State::Descending;

        loop {
            trace!(
                "state = {:?}, cube = {}, level = {}, trail = [{}]",
                state,
                DisplaySlice(&cube),
                self.decision_level(),
                zip_eq(variables, &cube)
                    .take(self.decision_level())
                    .map(|(&v, &s)| Lit::new(v, s))
                    .map(|lit| format!("{}@{}", lit, self.level(lit.var())))
                    .join(", ")
            );
            assert!(self.decision_level() <= variables.len());

            match state {
                State::Descending => {
                    if self.decision_level() == variables.len() {
                        trace!("Found valid cube: {}", DisplaySlice(&cube));
                        valid.push(
                            zip_eq(variables, &cube)
                                .take(self.decision_level())
                                .map(|(&v, &s)| Lit::new(v, s))
                                .collect_vec(),
                        );
                        total_count += 1;
                        state = State::Ascending;
                    } else {
                        self.assignment.new_decision_level();
                        let v = variables[self.decision_level() - 1];
                        let s = cube[self.decision_level() - 1];

                        let mut ok = true;
                        let current = trie.search(&cube[..self.decision_level()]);
                        if current == 0 {
                            ok = false;
                            state = State::Ascending;
                        }

                        if ok {
                            let p = Lit::new(v, s);
                            trace!("Trying to assign p = {} at new level {}", p, self.decision_level());
                            match self.value(p) {
                                LBool::True => {
                                    trace!("Literal {} already has True value", p);
                                    // do nothing
                                    // state = State::Descending;
                                }
                                LBool::False => {
                                    trace!(
                                        "Propagated different value for p = {} with trail = [{}]",
                                        p,
                                        zip_eq(variables, &cube)
                                            .take(self.decision_level())
                                            .map(|(&v, &s)| Lit::new(v, s))
                                            .map(|lit| format!("{}@{}", lit, self.level(lit.var())))
                                            .join(", ")
                                    );
                                    state = State::Ascending;
                                }
                                LBool::Undef => {
                                    trace!("Enqueueing {}", p);
                                    self.assignment.unchecked_enqueue(p, None);
                                    state = State::Propagating;
                                }
                            }
                        }
                    }
                }

                State::Ascending => {
                    assert!(self.decision_level() > 0);

                    // Find the 1-based index of the last 'false' value in 'cube':
                    let mut j = self.decision_level(); // 1-based
                    while j > 0 && cube[j - 1] {
                        j -= 1;
                    }
                    if j == 0 {
                        break;
                    }

                    // Increment the 'cube':
                    assert!(!cube[j - 1]);
                    cube[j - 1] = true;
                    for i in j..variables.len() {
                        cube[i] = false;
                    }

                    // Backtrack to the level before `j`:
                    self.backtrack(j - 1);

                    // Switch state to descending:
                    state = State::Descending;
                }

                State::Propagating => {
                    total_checked += 1;
                    if let Some(conflict) = self.propagate() {
                        trace!(
                            "Conflict {} for trail = [{}]",
                            self.clause(conflict),
                            zip_eq(variables, &cube)
                                .take(self.decision_level())
                                .map(|(&v, &s)| Lit::new(v, s))
                                .map(|lit| format!("{}@{}", lit, self.level(lit.var())))
                                .join(", ")
                        );
                        state = State::Ascending;
                    } else {
                        trace!(
                            "No conflict for trail = [{}]",
                            zip_eq(variables, &cube)
                                .take(self.decision_level())
                                .map(|(&v, &s)| Lit::new(v, s))
                                .map(|lit| format!("{}@{}", lit, self.level(lit.var())))
                                .join(", ")
                        );
                        state = State::Descending;
                    }
                }
            }
        }

        // Post-backtrack to zero level:
        self.backtrack(0);

        // Post-propagate:
        if let Some(_conflict) = self.propagate() {
            warn!("Conflict during post-propagation");
            self.ok = false;
            return 0;
        }

        debug!("Checked {} cubes, {} valid", total_checked, total_count);
        total_count
    }
}

// Additional methods.
impl Solver {
    pub fn add_clause_external<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<i32>,
    {
        let lits = lits.into_iter().map(|i| Lit::from_external(i.into())).collect_vec();
        self.add_clause(&lits);
    }

    pub fn solve_under_assumptions_external<I>(&mut self, assumptions: I) -> SolveResult
    where
        I: IntoIterator,
        I::Item: Into<i32>,
    {
        let assumptions = assumptions.into_iter().map(|i| Lit::from_external(i.into())).collect_vec();
        self.solve_under_assumptions(&assumptions)
    }
}

#[cfg(test)]
mod tests {
    use crate::trie::build_trie;

    use test_log::test;

    use super::*;

    #[test]
    fn test_correctness() {
        let mut solver = Solver::default();

        let tie = Lit::new(solver.new_var(), false);
        let shirt = Lit::new(solver.new_var(), false);
        info!("TIE = {:?}, SHIRT = {:?}", tie, shirt);
        solver.add_clause(&[-tie, shirt]);
        solver.add_clause(&[tie, shirt]);
        solver.add_clause(&[-tie, -shirt]);

        // Problem is satisfiable.
        let res = solver.solve();
        assert_eq!(res, SolveResult::Sat);

        // Check TIE is false, SHIRT is true.
        assert_eq!(solver.value(tie), LBool::False);
        assert_eq!(solver.value(shirt), LBool::True);

        // Assuming both TIE and SHIRT to be true.
        // Problem is unsatisfiable under assumptions.
        let response = solver.solve_under_assumptions(&[tie, shirt]);
        assert_eq!(response, SolveResult::Unsat);

        // `solve` resets assumptions, so calling it again should produce SAT.
        let response = solver.solve();
        assert_eq!(response, SolveResult::Sat);

        // Force TIE to true.
        solver.add_clause(&[tie]);

        // Problem is now finally unsatisfiable.
        let res = solver.solve();
        assert_eq!(res, SolveResult::Unsat);
    }

    #[test]
    fn test_auto_create_variables() {
        let mut solver = Solver::default();

        assert_eq!(solver.num_vars(), 0);
        solver.add_clause_external([-1, 2]);
        solver.add_clause_external([1, 2]);
        solver.add_clause_external([-1, -2]);
        assert_eq!(solver.num_vars(), 2);

        let res = solver.solve();
        assert_eq!(res, SolveResult::Sat);

        assert_eq!(solver.value(Lit::from_external(1)), LBool::False);
        assert_eq!(solver.value(Lit::from_external(2)), LBool::True);
    }

    #[test]
    fn test_propcheck() {
        let mut solver = Solver::default();

        let x1 = solver.new_lit();
        let x2 = solver.new_lit();
        let g1 = solver.new_lit();
        info!("x1 = {0:?} = {0}, x2 = {1:?} = {1}, g1 = {2:?} = {2}", x1, x2, g1);

        // g1 <=> x1 AND x2
        solver.add_clause(&[g1, -x1, -x2]);
        solver.add_clause(&[-g1, x1]);
        solver.add_clause(&[-g1, x2]);

        // Forbid (x1 AND ~x2)
        solver.add_clause(&[-x1, x2]);

        info!("vars = {}, clauses = {}", solver.num_vars(), solver.num_clauses());

        // Problem is satisfiable.
        // let res = solver.solve();
        // assert_eq!(res, SolveResult::Sat);

        let variables = vec![x1.var(), x2.var()];

        info!("----------------------");
        let count = solver.propcheck_all(&variables);
        info!("count = {}", count);

        info!("----------------------");
        let mut learnts = Vec::new();
        let count_tree = solver.propcheck_all_tree(&variables, 0, false, &mut learnts);
        info!("count_tree = {}", count_tree);

        assert_eq!(count, count_tree);

        info!("----------------------");
        let cubes = vec![vec![false, false], vec![true, true], vec![true, false], vec![false, true]];
        let trie = build_trie(&cubes);
        let mut valid = Vec::new();
        let count_trie = solver.propcheck_all_trie(&variables, &trie, &mut valid);
        info!("count_trie = {}", count_trie);

        assert_eq!(count, count_trie);
    }

    #[test]
    fn test_propcheck_tieshirt() {
        let mut solver = Solver::default();

        let tie = Lit::new(solver.new_var(), false);
        let shirt = Lit::new(solver.new_var(), false);
        info!("TIE = {:?}, SHIRT = {:?}", tie, shirt);

        solver.add_clause(&[-tie, shirt]);
        solver.add_clause(&[tie, shirt]);
        solver.add_clause(&[-tie, -shirt]);
        info!("vars = {}, clauses = {}", solver.num_vars(), solver.num_clauses());

        // Problem is satisfiable.
        // let res = solver.solve();
        // assert_eq!(res, SolveResult::Sat);

        let variables = vec![tie.var(), shirt.var()];
        let count = solver.propcheck_all(&variables);
        info!("count = {}", count);
        assert_eq!(count, 1);
    }
}
