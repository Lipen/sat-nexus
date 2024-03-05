use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;
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
    pub fn configure(&self, name: &'static str) {
        let c_string = CString::new(name).expect("CString::new failed");
        let res = unsafe { ccadical_configure(self.ptr, c_string.as_ptr()) };
        assert!(res);
    }

    /// Explicit version of setting an option.  If the option 'name' exists
    /// and 'val' can be parsed then 'true' is returned.  If the option value
    /// is out of range the actual value is computed as the closest (minimum or
    /// maximum) value possible, but still 'true' is returned.
    ///
    /// Options can only be set right after initialization.
    pub fn set_option(&self, name: &'static str, val: i32) {
        let c_string = CString::new(name).expect("CString::new failed");
        let ok = unsafe { ccadical_set_option(self.ptr, c_string.as_ptr(), val) };
        assert!(ok, "ccadical_set_option returned false");
    }

    /// Get the current value of the option 'name'.  If 'name' is invalid then
    /// zero is returned.  Here '--...' arguments as invalid options.
    pub fn get_option(&self, name: &'static str) -> i32 {
        let c_string = CString::new(name).expect("CString::new failed");
        unsafe { ccadical_get_option(self.ptr, c_string.as_ptr()) }
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
        let c_string = CString::new(name).expect("CString::new failed");
        let ok = unsafe { ccadical_limit(self.ptr, c_string.as_ptr(), limit) };
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
        match unsafe { ccadical_failed(self.ptr, lit) } {
            0 => Ok(false),
            1 => Ok(true),
            invalid => InvalidResponseFailedSnafu { lit, value: invalid }.fail(),
        }
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
        unsafe { ccadical_active_lit(self.ptr, lit) }
    }

    pub fn frozen(&self, lit: i32) -> Result<bool> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        match unsafe { ccadical_frozen(self.ptr, lit) } {
            0 => Ok(false),
            1 => Ok(true),
            invalid => InvalidResponseFrozenSnafu { lit, value: invalid }.fail(),
        }
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

    pub fn propcheck(&self, lits: &[i32], restore: bool) -> bool {
        unsafe {
            ccadical_propcheck_begin(self.ptr);
            for &lit in lits {
                assert_ne!(lit, 0);
                ccadical_propcheck_add(self.ptr, lit);
            }
            if restore {
                ccadical_propcheck(self.ptr)
            } else {
                ccadical_propcheck_no_restore(self.ptr)
            }
        }
    }

    pub fn propcheck_num_propagated(&self, lits: &[i32], restore: bool) -> (bool, u64) {
        unsafe {
            ccadical_propcheck_begin(self.ptr);
            for &lit in lits {
                assert_ne!(lit, 0);
                ccadical_propcheck_add(self.ptr, lit);
            }
            let mut num_propagated = 0;
            let res = if restore {
                ccadical_propcheck_num_propagated(self.ptr, &mut num_propagated)
            } else {
                ccadical_propcheck_num_propagated_no_restore(self.ptr, &mut num_propagated)
            };
            (res, num_propagated)
        }
    }

    pub fn propcheck_save_propagated(&self, lits: &[i32], restore: bool) -> (bool, Vec<i32>) {
        unsafe {
            ccadical_propcheck_begin(self.ptr);
            for &lit in lits {
                assert_ne!(lit, 0);
                ccadical_propcheck_add(self.ptr, lit);
            }
            let res = if restore {
                ccadical_propcheck_save_propagated(self.ptr)
            } else {
                ccadical_propcheck_save_propagated_no_restore(self.ptr)
            };
            let propagated_length = ccadical_propcheck_get_propagated_length(self.ptr);
            let mut propagated = Vec::with_capacity(propagated_length);
            ccadical_propcheck_get_propagated(self.ptr, propagated.as_mut_ptr());
            propagated.set_len(propagated_length);
            (res, propagated)
        }
    }

    pub fn propcheck_all_tree(&self, vars: &[i32], limit: u64) -> u64 {
        unsafe {
            ccadical_propcheck_all_tree_begin(self.ptr);
            for &v in vars {
                assert!(v > 0);
                ccadical_propcheck_all_tree_add(self.ptr, v);
            }
            ccadical_propcheck_all_tree(self.ptr, limit)
        }
    }

    pub fn propcheck_all_tree_valid(&self, vars: &[i32]) -> Vec<Vec<i32>> {
        unsafe {
            ccadical_propcheck_all_tree_begin(self.ptr);
            for &v in vars {
                assert!(v > 0);
                ccadical_propcheck_all_tree_add(self.ptr, v);
            }
            let res = ccadical_propcheck_all_tree_save_valid(self.ptr);
            let valid_length = ccadical_propcheck_all_tree_get_valid_length(self.ptr);
            assert_eq!(valid_length as u64, res);
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

    pub fn all_clauses_iter(&self) -> AllClausesIter {
        let length = unsafe { ccadical_build_all_clauses(self.ptr) };
        AllClausesIter {
            ptr: self.ptr,
            index: 0,
            length,
        }
    }
}

pub struct AllClausesIter {
    ptr: CCadicalPtr,
    index: usize,
    length: usize,
}

impl Iterator for AllClausesIter {
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
