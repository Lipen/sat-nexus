use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;
use snafu::ensure;

use ffi_utils::cstr2str;

use self::ffi::*;
use crate::common::*;

pub mod ffi {
    pub use cadical_sys::dynamic::*;
}

/// Cadical solver.
///
/// # Examples
///
/// ```
/// # fn main() -> color_eyre::Result<()> {
/// use cadical::*;
/// use cadical::dynamic::Cadical;
/// // Create solver
/// let solver = Cadical::new();
/// // Add some clauses: (a or b) and (b or c) and (not a or c) and (not c)
/// solver.add_clause([1, 2]);
/// solver.add_clause(vec![2, 3]);
/// solver.try_add_clause([-1, 3])?;
/// solver.add_clause([-3]);
/// // Solve the SAT problem
/// let response = solver.solve()?;
/// assert_eq!(response, SolveResponse::Sat);
/// // Query the result
/// assert_eq!(solver.val(1)?, LitValue::False);
/// assert_eq!(solver.val(2)?, LitValue::True);
/// assert_eq!(solver.val(3)?, LitValue::False);
/// # Ok(())
/// # }
/// ```
pub struct Cadical {
    ffi: &'static CCadicalFFI,
    ptr: CCadicalPtr,
}

impl Cadical {
    pub fn new() -> Self {
        Self::new_custom(CCadicalFFI::instance())
    }

    pub fn new_custom(ffi: &'static CCadicalFFI) -> Self {
        Cadical {
            ffi,
            ptr: unsafe { ffi.ccadical_init() },
        }
    }
}

impl Default for Cadical {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Cadical {
    fn drop(&mut self) {
        self.release()
    }
}

impl Debug for Cadical {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cadical").field("ptr", &self.ptr).finish()
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
        unsafe { cstr2str(self.ffi.ccadical_signature()) }
    }

    pub fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe { self.ffi.ccadical_release(self.ptr) }
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
        unsafe { self.ffi.ccadical_constrain(self.ptr, lit_or_zero) }
    }

    /// Determine whether the constraint was used to proof the unsatisfiability.
    /// Note that the formula might still be unsatisfiable without the constraint.
    pub fn constraint_failed(&self) -> Result<bool> {
        match unsafe { self.ffi.ccadical_constraint_failed(self.ptr) } {
            0 => Ok(false),
            1 => Ok(true),
            invalid => InvalidResponseConstraintFailedSnafu { value: invalid }.fail(),
        }
    }

    // Overwrite (some) options with the forced values of the configuration.
    // The result is 'true' iff the 'name' is a valid configuration.
    pub fn configure(&self, name: &'static str) {
        let c_string = CString::new(name).expect("CString::new failed");
        let ok = unsafe { self.ffi.ccadical_configure(self.ptr, c_string.as_ptr()) };
        assert!(ok, "ccadical_configure returned false");
    }

    /// Explicit version of setting an option.  If the option 'name' exists
    /// and 'val' can be parsed then 'true' is returned.  If the option value
    /// is out of range the actual value is computed as the closest (minimum or
    /// maximum) value possible, but still 'true' is returned.
    ///
    /// Options can only be set right after initialization.
    pub fn set_option(&self, name: &'static str, val: i32) {
        let c_string = CString::new(name).expect("CString::new failed");
        let ok = unsafe { self.ffi.ccadical_set_option(self.ptr, c_string.as_ptr(), val) };
        assert!(ok, "ccadical_set_option returned false");
    }

    /// Get the current value of the option 'name'.  If 'name' is invalid then
    /// zero is returned.  Here '--...' arguments as invalid options.
    pub fn get_option(&self, name: &'static str) -> i32 {
        let c_string = CString::new(name).expect("CString::new failed");
        unsafe { self.ffi.ccadical_get_option(self.ptr, c_string.as_ptr()) }
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
        let ok = unsafe { self.ffi.ccadical_limit(self.ptr, c_string.as_ptr(), limit) };
        assert!(ok, "ccadical_limit returned false");
    }

    /// Add valid literal to clause or zero to terminate clause.
    pub fn add(&self, lit_or_zero: i32) {
        unsafe { self.ffi.ccadical_add(self.ptr, lit_or_zero) }
    }

    /// Assume valid non zero literal for next call to 'solve'.
    /// These assumptions are reset after the call to 'solve'
    /// as well as after returning from 'simplify' and 'lookahead'.
    pub fn assume(&self, lit: i32) -> Result<()> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        unsafe { self.ffi.ccadical_assume(self.ptr, lit) }
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
        match unsafe { self.ffi.ccadical_simplify(self.ptr) } {
            0 => Ok(SimplifyResponse::Unknown),
            10 => Ok(SimplifyResponse::Sat),
            20 => Ok(SimplifyResponse::Unsat),
            invalid => InvalidResponseSimplifySnafu { value: invalid }.fail(),
        }
    }

    /// Try to solve the current formula.
    pub fn solve(&self) -> Result<SolveResponse> {
        match unsafe { self.ffi.ccadical_solve(self.ptr) } {
            0 => Ok(SolveResponse::Interrupted),
            10 => Ok(SolveResponse::Sat),
            20 => Ok(SolveResponse::Unsat),
            invalid => InvalidResponseSolveSnafu { value: invalid }.fail(),
        }
    }

    /// Force termination of 'solve' asynchronously.
    pub fn terminate(&self) {
        unsafe { self.ffi.ccadical_terminate(self.ptr) }
    }

    /// Get value of valid non-zero literal.
    pub fn val(&self, lit: i32) -> Result<LitValue> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        match unsafe { self.ffi.ccadical_val(self.ptr, lit) } {
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
        let res = unsafe { self.ffi.ccadical_failed(self.ptr, lit) };
        Ok(res)
    }

    pub fn print_statistics(&self) {
        unsafe { self.ffi.ccadical_print_statistics(self.ptr) }
    }

    /// Number of variables.
    pub fn vars(&self) -> i64 {
        unsafe { self.ffi.ccadical_vars(self.ptr) }
    }

    /// Number of active variables.
    pub fn active(&self) -> i64 {
        unsafe { self.ffi.ccadical_active(self.ptr) }
    }

    /// Number of active irredundant clauses.
    pub fn irredundant(&self) -> i64 {
        unsafe { self.ffi.ccadical_irredundant(self.ptr) }
    }

    /// Number of conflicts.
    pub fn conflicts(&self) -> i64 {
        unsafe { self.ffi.ccadical_conflicts(self.ptr) }
    }

    /// Number of decisions.
    pub fn decisions(&self) -> i64 {
        unsafe { self.ffi.ccadical_decisions(self.ptr) }
    }

    /// Number of restarts.
    pub fn restarts(&self) -> i64 {
        unsafe { self.ffi.ccadical_restarts(self.ptr) }
    }

    /// Number of propagations.
    pub fn propagations(&self) -> i64 {
        unsafe { self.ffi.ccadical_propagations(self.ptr) }
    }

    /// Root level assigned variables can be queried with this function.
    /// It returns '1' if the literal is implied by the formula, '-1' if its
    /// negation is implied, or '0' if this is unclear at this point.
    pub fn fixed(&self, lit: i32) -> Result<FixedResponse> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        match unsafe { self.ffi.ccadical_fixed(self.ptr, lit) } {
            1 => Ok(FixedResponse::Positive),
            -1 => Ok(FixedResponse::Negative),
            0 => Ok(FixedResponse::Unclear),
            invalid => InvalidResponseFixedSnafu { lit, value: invalid }.fail(),
        }
    }

    pub fn frozen(&self, lit: i32) -> Result<bool> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        let res = unsafe { self.ffi.ccadical_frozen(self.ptr, lit) };
        Ok(res)
    }

    pub fn freeze(&self, lit: i32) -> Result<()> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        unsafe { self.ffi.ccadical_freeze(self.ptr, lit) }
        Ok(())
    }

    pub fn melt(&self, lit: i32) -> Result<()> {
        ensure!(lit != 0, ZeroLiteralSnafu);
        unsafe { self.ffi.ccadical_melt(self.ptr, lit) }
        Ok(())
    }
}

/// Additional methods.
impl Cadical {
    pub fn reset(&mut self) {
        self.release();
        self.ptr = unsafe { self.ffi.ccadical_init() };
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
