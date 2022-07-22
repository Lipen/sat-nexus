use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};

use crate::ffi::*;
use crate::types::*;

/// Cadical solver.
///
/// # Examples
///
/// ```
/// # fn main() -> color_eyre::eyre::Result<()> {
/// use cadical::{Cadical, SolveResponse};
/// // Create solver
/// use cadical::LitValue;
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
        Cadical { ffi, ptr: ffi.init() }
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
        self.ffi.signature()
    }

    pub fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe { self.ffi.ccadical_release(self.ptr) }
            self.ptr = std::ptr::null_mut();
        }
    }

    /// Explicit version of setting an option.  If the option 'name' exists
    /// and 'val' can be parsed then 'true' is returned.  If the option value
    /// is out of range the actual value is computed as the closest (minimum or
    /// maximum) value possible, but still 'true' is returned.
    ///
    /// Options can only be set right after initialization.
    pub fn set_option(&self, name: &'static str, val: i32) {
        let c_string = CString::new(name).expect("CString::new failed");
        unsafe { self.ffi.ccadical_set_option(self.ptr, c_string.as_ptr(), val) }
    }

    /// Get the current value of the option 'name'.  If 'name' is invalid then
    /// zero is returned.  Here '--...' arguments as invalid options.
    pub fn get_option(&self, name: &'static str) -> i32 {
        let c_string = CString::new(name).expect("CString::new failed");
        let c_chars = c_string.as_ptr();
        unsafe { self.ffi.ccadical_get_option(self.ptr, c_chars) }
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
        let c_chars = c_string.as_ptr();
        unsafe { self.ffi.ccadical_limit(self.ptr, c_chars, limit) }
    }

    /// Add valid literal to clause or zero to terminate clause.
    pub fn add(&self, lit_or_zero: i32) {
        unsafe { self.ffi.ccadical_add(self.ptr, lit_or_zero) }
    }

    /// Assume valid non zero literal for next call to 'solve'.
    /// These assumptions are reset after the call to 'solve'
    /// as well as after returning from 'simplify' and 'lookahead'.
    pub fn assume(&self, lit: i32) {
        debug_assert_ne!(lit, 0, "Literal must be non-zero");
        // ensure!(lit != 0, ZeroLiteral);
        unsafe { self.ffi.ccadical_assume(self.ptr, lit) }
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
            invalid => Err(CadicalError::InvalidResponseSimplify { value: invalid }),
        }
    }

    /// Try to solve the current formula.
    pub fn solve(&self) -> Result<SolveResponse> {
        match unsafe { self.ffi.ccadical_solve(self.ptr) } {
            0 => Ok(SolveResponse::Interrupted),
            10 => Ok(SolveResponse::Sat),
            20 => Ok(SolveResponse::Unsat),
            invalid => Err(CadicalError::InvalidResponseSolve { value: invalid }),
        }
    }

    /// Force termination of 'solve' asynchronously.
    pub fn terminate(&self) {
        unsafe { self.ffi.ccadical_terminate(self.ptr) }
    }

    /// Get value of valid non-zero literal.
    pub fn val(&self, lit: i32) -> Result<LitValue> {
        debug_assert_ne!(lit, 0, "Literal must be non-zero");
        // ensure!(lit != 0, ZeroLiteral);
        match unsafe { self.ffi.ccadical_val(self.ptr, lit) } {
            p if p == lit => Ok(LitValue::True),
            n if n == -lit => Ok(LitValue::False),
            invalid => Err(CadicalError::InvalidResponseVal { lit, value: invalid }),
        }
    }

    /// Determine whether the valid non-zero literal is in the core.
    /// Returns `true` if the literal is in the core and `false` otherwise.
    /// Note that the core does not have to be minimal.
    pub fn failed(&self, lit: i32) -> Result<bool> {
        debug_assert_ne!(lit, 0, "Literal must be non-zero");
        // ensure!(lit != 0, ZeroLiteral);
        match unsafe { self.ffi.ccadical_failed(self.ptr, lit) } {
            0 => Ok(false),
            1 => Ok(true),
            invalid => Err(CadicalError::InvalidResponseFailed { lit, value: invalid }),
        }
    }

    pub fn print_statistics(&self) {
        unsafe { self.ffi.ccadical_print_statistics(self.ptr) }
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
        debug_assert_ne!(lit, 0, "Literal must be non-zero");
        // ensure!(lit != 0, ZeroLiteral);
        match unsafe { self.ffi.ccadical_fixed(self.ptr, lit) } {
            1 => Ok(FixedResponse::Implied),
            -1 => Ok(FixedResponse::Negation),
            0 => Ok(FixedResponse::Unclear),
            invalid => Err(CadicalError::InvalidResponseFixed { lit, value: invalid }),
        }
    }

    pub fn frozen(&self, lit: i32) -> Result<FrozenResponse> {
        debug_assert_ne!(lit, 0, "Literal must be non-zero");
        // ensure!(lit != 0, ZeroLiteral);
        match unsafe { self.ffi.ccadical_frozen(self.ptr, lit) } {
            0 => Ok(FrozenResponse::NotFrozen),
            1 => Ok(FrozenResponse::Frozen),
            invalid => Err(CadicalError::InvalidResponseFrozen { lit, value: invalid }),
        }
    }

    pub fn freeze(&self, lit: i32) {
        debug_assert_ne!(lit, 0, "Literal must be non-zero");
        // ensure!(lit != 0, ZeroLiteral);
        unsafe { self.ffi.ccadical_freeze(self.ptr, lit) }
    }

    pub fn melt(&self, lit: i32) {
        debug_assert_ne!(lit, 0, "Literal must be non-zero");
        // ensure!(lit != 0, ZeroLiteral);
        unsafe { self.ffi.ccadical_melt(self.ptr, lit) }
    }
}

/// Additional methods.
impl Cadical {
    pub fn reset(&mut self) {
        self.release();
        self.ptr = self.ffi.init();
    }

    pub fn add_clause<I, L>(&self, lits: I)
    where
        I: IntoIterator<Item = L>,
        L: Into<i32>,
    {
        for lit in lits.into_iter() {
            self.add(lit.into());
        }
        self.add(0);
    }

    pub fn try_add_clause<I, L>(&self, lits: I) -> Result<(), <L as TryInto<i32>>::Error>
    where
        I: IntoIterator<Item = L>,
        L: TryInto<i32>,
    {
        let lits: Vec<i32> = lits.into_iter().map(|x| x.try_into()).collect::<Result<_, _>>()?;
        self.add_clause(lits);
        Ok(())
    }
}
