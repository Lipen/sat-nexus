use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;

use itertools::{zip_eq, Itertools};
use log::{debug, trace};
use snafu::ensure;

use cadical_sys::statik::*;
use ffi_utils::cstr2str;

use crate::common::*;

#[derive(Debug)]
pub struct Cadical {
    ptr: CCadicalPtr,
}

impl Cadical {
    pub fn new() -> Self {
        let ptr = unsafe { ccadical_init() };
        Self { ptr }
    }
}

impl Default for Cadical {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Cadical {
    fn drop(&mut self) {
        unsafe { ccadical_release(self.ptr) }
    }
}

impl Display for Cadical {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.signature())
    }
}

/// Cadical interface.
impl Cadical {
    pub fn signature(&self) -> &'static str {
        unsafe { cstr2str(ccadical_signature()) }
    }

    pub fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ccadical_release(self.ptr) }
            self.ptr = std::ptr::null_mut();
        }
    }

    /// Adds a literal to the constraint clause. Same functionality as 'add' but
    /// the clause only exists for the next call to solve (same lifetime as
    /// assumptions). Only one constraint may exists at a time. A new constraint
    /// replaces the old.
    /// The main application of this functionality is the model checking algorithm
    /// IC3. See our FMCAD'21 paper `[FroleyksBiere-FMCAD'21]` for more details.
    ///
    /// Add valid literal to the constraint clause or zero to terminate it.
    pub fn constrain(&self, lit_or_zero: i32) {
        unsafe { ccadical_constrain(self.ptr, lit_or_zero) }
    }

    /// Determine whether the constraint was used to proof the unsatisfiability.
    /// Note that the formula might still be unsatisfiable without the constraint.
    pub fn constraint_failed(&self) -> Result<bool> {
        match unsafe { ccadical_constraint_failed(self.ptr) } {
            0 => Ok(false),
            1 => Ok(true),
            invalid => InvalidResponseConstraintFailedSnafu { value: invalid }.fail(),
        }
    }

    // Overwrite (some) options with the forced values of the configuration.
    // The result is 'true' iff the 'name' is a valid configuration.
    pub fn configure(&self, name: &str) {
        let name = CString::new(name).expect("CString::new failed");
        let res = unsafe { ccadical_configure(self.ptr, name.as_ptr()) };
        assert!(res);
    }

    /// Explicit version of setting an option.  If the option 'name' exists
    /// and 'val' can be parsed then 'true' is returned.  If the option value
    /// is out of range the actual value is computed as the closest (minimum or
    /// maximum) value possible, but still 'true' is returned.
    ///
    /// Options can only be set right after initialization.
    pub fn set_option(&self, name: &str, val: i32) {
        let name = CString::new(name).expect("CString::new failed");
        let ok = unsafe { ccadical_set_option(self.ptr, name.as_ptr(), val) };
        assert!(ok, "ccadical_set_option returned false");
    }

    /// Get the current value of the option 'name'.  If 'name' is invalid then
    /// zero is returned.  Here '--...' arguments as invalid options.
    pub fn get_option(&self, name: &str) -> i32 {
        let name = CString::new(name).expect("CString::new failed");
        unsafe { ccadical_get_option(self.ptr, name.as_ptr()) }
    }

    /// Specify search limits, where currently 'name' can be "conflicts",
    /// "decisions", "preprocessing", or "localsearch".  The first two limits
    /// are unbounded by default.  Thus using a negative limit for conflicts or
    /// decisions switches back to the default of unlimited search (for that
    /// particular limit).  The preprocessing limit determines the number of
    /// preprocessing rounds, which is zero by default.  Similarly, the local
    /// search limit determines the number of local search rounds (also zero by
    /// default).  As with 'set', the return value denotes whether the limit
    /// 'name' is valid.  These limits are only valid for the next 'solve' or
    /// 'simplify' call and reset to their default after 'solve' returns (as
    /// well as overwritten and reset during calls to 'simplify' and
    /// 'lookahead').  We actually also have an internal "terminate" limit
    /// which however should only be used for testing and debugging.
    pub fn limit(&self, name: &str, limit: i32) {
        let name = CString::new(name).expect("CString::new failed");
        let ok = unsafe { ccadical_limit(self.ptr, name.as_ptr(), limit) };
        assert!(ok, "ccadical_limit returned false");
    }

    /// Add valid literal to clause or zero to terminate clause.
    pub fn add(&self, lit_or_zero: i32) {
        unsafe { ccadical_add(self.ptr, lit_or_zero) }
    }

    /// Assume valid non zero literal for next call to 'solve'.
    /// These assumptions are reset after the call to 'solve'
    /// as well as after returning from 'simplify' and 'lookahead'.
    pub fn assume(&self, lit: i32) -> Result<()> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        unsafe { ccadical_assume(self.ptr, lit) }
        Ok(())
    }

    /// This function executes the given number of preprocessing rounds.
    /// It is similar to 'solve' with 'limits ("preprocessing", rounds)'
    /// except that no CDCL nor local search, nor lucky phases are executed.
    /// The result values are also the same: 0=unknown, 10=satisfiable, 20=unsatisfiable.
    /// As 'solve' it resets current assumptions and limits before returning.
    ///
    /// Internally, the default number of rounds is 3.
    pub fn simplify(&self) -> Result<SimplifyResponse> {
        match unsafe { ccadical_simplify(self.ptr) } {
            0 => Ok(SimplifyResponse::Unknown),
            10 => Ok(SimplifyResponse::Sat),
            20 => Ok(SimplifyResponse::Unsat),
            invalid => InvalidResponseSimplifySnafu { value: invalid }.fail(),
        }
    }

    /// Try to solve the current formula.
    pub fn solve(&self) -> Result<SolveResponse> {
        match unsafe { ccadical_solve(self.ptr) } {
            0 => Ok(SolveResponse::Interrupted),
            10 => Ok(SolveResponse::Sat),
            20 => Ok(SolveResponse::Unsat),
            invalid => InvalidResponseSolveSnafu { value: invalid }.fail(),
        }
    }

    /// Force termination of 'solve' asynchronously.
    pub fn terminate(&self) {
        unsafe { ccadical_terminate(self.ptr) }
    }

    pub fn reset_assumptions(&self) {
        unsafe { ccadical_reset_assumptions(self.ptr) }
    }

    pub fn reset_constraint(&self) {
        unsafe { ccadical_reset_constraint(self.ptr) }
    }

    /// Get value of valid non-zero literal.
    pub fn val(&self, lit: i32) -> Result<LitValue> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        match unsafe { ccadical_val(self.ptr, lit) } {
            p if p == lit => Ok(LitValue::True),
            n if n == -lit => Ok(LitValue::False),
            invalid => InvalidResponseValSnafu { lit, value: invalid }.fail(),
        }
    }

    /// Determine whether the valid non-zero literal is in the core.
    /// Returns `true` if the literal is in the core and `false` otherwise.
    /// Note that the core does not have to be minimal.
    pub fn failed(&self, lit: i32) -> Result<bool> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        Ok(unsafe { ccadical_failed(self.ptr, lit) })
    }

    /// Triggers the conclusion of incremental proofs.
    /// if the solver is SATISFIED it will trigger extend ()
    /// and give the model to the proof tracer through conclude_sat ()
    /// if the solver is UNSATISFIED it will trigger failing ()
    /// which will learn new clauses as explained below:
    /// In case of failed assumptions will provide a core negated
    /// as a clause through the proof tracer interface.
    /// With a failing constraint these can be multiple clauses.
    /// Then it will trigger a conclude_unsat event with the id(s)
    /// of the newly learnt clauses or the id of the global conflict.
    pub fn conclude(&self) {
        unsafe { ccadical_conclude(self.ptr) }
    }

    pub fn trace_proof<P>(&self, path: P)
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path = path.to_str().expect("path is not valid UTF-8");
        let path = CString::new(path).expect("CString::new failed");
        let ok = unsafe { ccadical_trace_proof(self.ptr, path.as_ptr()) };
        assert!(ok, "ccadical_trace_proof returned false");
    }

    pub fn close_proof(&self) {
        unsafe { ccadical_close_proof(self.ptr) }
    }

    pub fn read_dimacs<P>(&self, path: P, strict: i32)
    where
        P: AsRef<Path>,
    {
        assert!(0 <= strict && strict <= 2);
        let path = path.as_ref();
        let path = path.to_str().expect("path is not valid UTF-8");
        let path = CString::new(path).expect("CString::new failed");
        unsafe { ccadical_read_dimacs(self.ptr, path.as_ptr(), strict) }
    }

    pub fn write_dimacs<P>(&self, path: P)
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path = path.to_str().expect("path is not valid UTF-8");
        let path = CString::new(path).expect("CString::new failed");
        unsafe { ccadical_write_dimacs(self.ptr, path.as_ptr()) }
    }

    pub fn print_statistics(&self) {
        unsafe { ccadical_print_statistics(self.ptr) }
    }

    /// Number of variables.
    pub fn vars(&self) -> i64 {
        unsafe { ccadical_vars(self.ptr) }
    }

    /// Number of active variables.
    pub fn active(&self) -> i64 {
        unsafe { ccadical_active(self.ptr) }
    }

    /// Number of active redundant clauses.
    pub fn redundant(&self) -> i64 {
        unsafe { ccadical_redundant(self.ptr) }
    }

    /// Number of active irredundant clauses.
    pub fn irredundant(&self) -> i64 {
        unsafe { ccadical_irredundant(self.ptr) }
    }

    /// Number of conflicts.
    pub fn conflicts(&self) -> i64 {
        unsafe { ccadical_conflicts(self.ptr) }
    }

    /// Number of decisions.
    pub fn decisions(&self) -> i64 {
        unsafe { ccadical_decisions(self.ptr) }
    }

    /// Number of restarts.
    pub fn restarts(&self) -> i64 {
        unsafe { ccadical_restarts(self.ptr) }
    }

    /// Number of propagations.
    pub fn propagations(&self) -> i64 {
        unsafe { ccadical_propagations(self.ptr) }
    }

    /// Root level assigned variables can be queried with this function.
    /// It returns '1' if the literal is implied by the formula, '-1' if its
    /// negation is implied, or '0' if this is unclear at this point.
    pub fn fixed(&self, lit: i32) -> Result<FixedResponse> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        match unsafe { ccadical_fixed(self.ptr, lit) } {
            1 => Ok(FixedResponse::Positive),
            -1 => Ok(FixedResponse::Negative),
            0 => Ok(FixedResponse::Unclear),
            invalid => InvalidResponseFixedSnafu { lit, value: invalid }.fail(),
        }
    }

    pub fn is_active(&self, lit: i32) -> bool {
        unsafe { ccadical_is_active(self.ptr, lit) }
    }

    pub fn frozen(&self, lit: i32) -> Result<bool> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        Ok(unsafe { ccadical_frozen(self.ptr, lit) })
    }

    pub fn freeze(&self, lit: i32) -> Result<()> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        unsafe { ccadical_freeze(self.ptr, lit) }
        Ok(())
    }

    pub fn melt(&self, lit: i32) -> Result<()> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        unsafe { ccadical_melt(self.ptr, lit) }
        Ok(())
    }

    pub fn internal_propagate(&self) -> bool {
        unsafe { ccadical_internal_propagate(self.ptr) }
    }

    pub fn internal_reset_conflict(&self) {
        unsafe { ccadical_internal_reset_conflict(self.ptr) }
    }

    pub fn internal_level(&self) -> usize {
        unsafe { ccadical_internal_level(self.ptr) as usize }
    }

    pub fn internal_val(&self, lit: i32) -> i8 {
        assert_ne!(lit, 0);
        unsafe { ccadical_internal_val(self.ptr, lit) }
    }

    pub fn internal_assume_decision(&self, lit: i32) {
        // Note: lit can be 0, which creates a "dummy" level without a decision.
        unsafe {
            ccadical_internal_assume_decision(self.ptr, lit);
        }
    }

    pub fn internal_backtrack(&self, new_level: usize) {
        unsafe {
            ccadical_internal_backtrack(self.ptr, new_level as i32);
        }
    }

    pub fn propcheck(&self, lits: &[i32], restore: bool, save_propagated: bool, save_core: bool) -> (bool, u64) {
        unsafe {
            ccadical_propcheck_begin(self.ptr);
            for &lit in lits {
                assert_ne!(lit, 0);
                ccadical_propcheck_add(self.ptr, lit);
            }
            let mut num_propagated = 0;
            let res = ccadical_propcheck(self.ptr, restore, &mut num_propagated, save_propagated, save_core);
            (res, num_propagated)
        }
    }

    pub fn propcheck_get_propagated(&self) -> Vec<i32> {
        unsafe {
            let propagated_length = ccadical_propcheck_get_propagated_length(self.ptr);
            let mut propagated = Vec::with_capacity(propagated_length);
            ccadical_propcheck_get_propagated(self.ptr, propagated.as_mut_ptr());
            propagated.set_len(propagated_length);
            propagated
        }
    }

    pub fn propcheck_get_core(&self) -> Vec<i32> {
        unsafe {
            let core_length = ccadical_propcheck_get_core_length(self.ptr);
            let mut core = Vec::with_capacity(core_length);
            ccadical_propcheck_get_core(self.ptr, core.as_mut_ptr());
            core.set_len(core_length);
            core
        }
    }

    pub fn propcheck_all_tree(&self, vars: &[i32], limit: u64, save: bool) -> u64 {
        unsafe {
            ccadical_propcheck_all_tree_begin(self.ptr);
            for &v in vars {
                assert!(v > 0);
                ccadical_propcheck_all_tree_add(self.ptr, v);
            }
            ccadical_propcheck_all_tree(self.ptr, limit, save)
        }
    }

    pub fn propcheck_all_tree_get_valid(&self) -> Vec<Vec<i32>> {
        unsafe {
            let valid_length = ccadical_propcheck_all_tree_get_valid_length(self.ptr);
            let mut valid = Vec::with_capacity(valid_length);
            for i in 0..valid_length {
                let cube_length = ccadical_propcheck_all_tree_get_cube_length(self.ptr, i);
                let mut cube = Vec::with_capacity(cube_length);
                ccadical_propcheck_all_tree_get_cube(self.ptr, i, cube.as_mut_ptr());
                cube.set_len(cube_length);
                valid.push(cube);
            }
            valid
        }
    }

    pub fn clauses_iter(&self) -> ClausesIter {
        unsafe { ClausesIter::new(self.ptr, false) }
    }

    pub fn all_clauses_iter(&self) -> ClausesIter {
        unsafe { ClausesIter::new(self.ptr, true) }
    }
}

pub struct ClausesIter {
    ptr: CCadicalPtr,
    length: usize,
    index: usize,
}

impl ClausesIter {
    pub unsafe fn new(ptr: CCadicalPtr, redundant: bool) -> Self {
        let length = ccadical_traverse_clauses(ptr, redundant);
        Self { ptr, length, index: 0 }
    }
}

impl Drop for ClausesIter {
    fn drop(&mut self) {
        unsafe { ccadical_clear_clauses(self.ptr) }
    }
}

impl Iterator for ClausesIter {
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            unsafe {
                let clause_length = ccadical_get_clause_length(self.ptr, self.index);
                let mut clause = Vec::with_capacity(clause_length);
                ccadical_get_clause(self.ptr, self.index, clause.as_mut_ptr());
                clause.set_len(clause_length);
                self.index += 1;
                Some(clause)
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.length - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ClausesIter {
    fn len(&self) -> usize {
        self.length
    }
}

/// Additional methods.
impl Cadical {
    pub fn reset(&mut self) {
        self.release();
        self.ptr = unsafe { ccadical_init() };
    }

    pub fn add_clause<I>(&self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<i32>,
    {
        for lit in lits.into_iter() {
            self.add(lit.into());
        }
        self.add(0);
    }

    // TODO: remove
    pub fn try_add_clause<I>(&self, lits: I) -> Result<(), <I::Item as TryInto<i32>>::Error>
    where
        I: IntoIterator,
        I::Item: TryInto<i32>,
    {
        let lits: Vec<i32> = lits.into_iter().map(|x| x.try_into()).try_collect()?;
        self.add_clause(lits);
        Ok(())
    }
}

impl Cadical {
    pub fn propcheck_all_tree_via_internal(
        &self,
        vars: &[i32],
        limit: u64,
        mut out_valid: Option<&mut Vec<Vec<i32>>>,
        mut out_invalid: Option<&mut Vec<Vec<i32>>>,
    ) -> u64 {
        assert!(vars.len() < 30);

        // TODO:
        // if (internal->unsat || internal->unsat_constraint) {
        //     std::cout << "Already unsat" << std::endl;
        //     return 0;
        // }

        // Trivial case:
        if vars.is_empty() {
            return 0;
        }

        // Backtrack to 0 level before prop-checking:
        if self.internal_level() > 0 {
            trace!("Backtracking from level {} to 0", self.internal_level());
            self.internal_backtrack(0);
        }

        // Propagate everything that needs to be propagated:
        if !self.internal_propagate() {
            debug!("Conflict during pre-propagation");
            self.internal_reset_conflict();
            return 0;
        }

        // Freeze variables:
        for &v in vars.iter() {
            self.freeze(v).unwrap()
        }

        let mut cube = vec![-1; vars.len()];
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
            let level = self.internal_level();
            assert!(level <= vars.len());

            match state {
                State::Descending => {
                    if level == vars.len() {
                        if let Some(valid) = &mut out_valid {
                            valid.push(zip_eq(vars, &cube).map(|(&v, &s)| v * s).collect());
                        }
                        total_count += 1;
                        if limit > 0 && total_count >= limit {
                            trace!("reached the limit: {} >= {}", total_count, limit);
                            break;
                        }
                        state = State::Ascending;
                    } else {
                        let lit = vars[level] * cube[level];
                        let b = self.internal_val(lit);
                        if b > 0 {
                            // Dummy level:
                            self.internal_assume_decision(0);
                            state = State::Descending;
                        } else if b < 0 {
                            // // Conflicting assignment:
                            // debug!(
                            //     "Conflicting assignment of {} on level {} for cube = [{}]",
                            //     lit,
                            //     level + 1,
                            //     zip_eq(vars, &cube)
                            //         .take(level + 1)
                            //         .map(|(&v, &s)| v * s)
                            //         .map(|lit| format!("{}", lit))
                            //         .join(", ")
                            // );
                            if let Some(invalid) = &mut out_invalid {
                                // TODO: extract core somehow
                                invalid.push(
                                    zip_eq(vars, &cube)
                                        .take(level + 1)
                                        .map(|(&v, &s)| v * s)
                                        // TODO: .filter(...)
                                        .collect(),
                                );
                            }
                            // Dummy level:
                            self.internal_assume_decision(0);
                            state = State::Ascending;
                        } else {
                            // Enqueue the literal:
                            self.internal_assume_decision(lit);
                            state = State::Propagating;
                        }
                    }
                }

                State::Ascending => {
                    assert!(level > 0);

                    // Find the 1-based index of the last 'false' value in 'cube':
                    let mut i = level; // 1-based
                    while i > 0 && cube[i - 1] > 0 {
                        i -= 1;
                    }
                    if i == 0 {
                        break;
                    }

                    // Increment the 'cube':
                    assert_eq!(cube[i - 1], -1);
                    cube[i - 1] = 1;
                    for j in i..vars.len() {
                        cube[j] = -1;
                    }

                    // Backtrack to the level before `i`:
                    self.internal_backtrack(i - 1);

                    // Switch state to descending:
                    state = State::Descending;
                }

                State::Propagating => {
                    total_checked += 1;
                    if !self.internal_propagate() {
                        // Conflict.
                        // debug!(
                        //     "Conflict on level {} for cube = [{}]",
                        //     level,
                        //     zip_eq(vars, &cube)
                        //         .take(level)
                        //         .map(|(&v, &s)| v * s)
                        //         .map(|lit| format!("{}", lit))
                        //         .join(", ")
                        // );
                        if let Some(invalid) = &mut out_invalid {
                            invalid.push(
                                zip_eq(vars, &cube)
                                    .take(level)
                                    .map(|(&v, &s)| v * s)
                                    // .filter(|&lit| self.internal_failed(lit))
                                    .collect(),
                            );
                        }
                        self.internal_reset_conflict();
                        state = State::Ascending;
                    } else {
                        // No conflict.
                        state = State::Descending;
                    }
                }
            }
        }

        // Post-backtrack to zero level:
        self.internal_backtrack(0);

        // Melt variables:
        for &v in vars.iter() {
            self.melt(v).unwrap()
        }

        trace!("Checked {} cubes, found {} valid", total_checked, total_count);
        total_count
    }
}

impl Cadical {
    pub fn add_unit_clause(&self, lit: i32) {
        assert_ne!(lit, 0);
        unsafe {
            ccadical_add_unit_clause(self.ptr, lit);
        }
    }

    pub fn add_derived(&self, lit_or_zero: i32) {
        unsafe { ccadical_add_derived(self.ptr, lit_or_zero) }
    }

    pub fn add_derived_clause<I>(&self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<i32>,
    {
        let clause = lits.into_iter().map(|lit| lit.into()).collect_vec();

        self.internal_backtrack(0);
        let res = self.internal_propagate();
        assert!(res);

        if clause.len() >= 2 {
            for lit in clause {
                assert!(self.is_active(lit), "lit {} is not active", lit);
                self.add_derived(lit);
            }
            self.add_derived(0);
        } else {
            let lit = clause[0];
            if self.is_active(lit) {
                self.add_unit_clause(lit);
                assert!(!self.is_active(lit));
            } else {
                log::warn!("unit {} is not active", lit);
            }
        }

        let res = self.internal_propagate();
        assert!(res);
    }
}
