use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;

use super::ffi::*;
use super::lbool::*;
use super::lit::*;
use super::var::*;

/// MiniSat solver.
///
/// # Examples
///
/// ```
/// # fn main() -> color_eyre::Result<()> {
/// use minisat::dynamic::*;
/// // Create solver
/// let solver = MiniSat::new();
/// // Initialize variables
/// let a = solver.new_lit();
/// let b = solver.new_lit();
/// let c = solver.new_lit();
/// // Add some clauses: (a or b) and (b or c) and (not a or c) and (not c)
/// solver.add_clause([a, b]);
/// solver.add_clause(vec![b, c]);
/// solver.try_add_clause([-a, c])?;
/// solver.add_clause([-c]);
/// // Solve the SAT problem
/// assert!(solver.solve());
/// // Query the result
/// assert_eq!(solver.model_value_lit(a), LBool::False);
/// assert_eq!(solver.model_value_lit(b), LBool::True);
/// assert_eq!(solver.model_value_lit(c), LBool::False);
/// # Ok(())
/// # }
/// ```
pub struct MiniSat {
    ffi: &'static CMiniSatFFI,
    ptr: CMiniSatPtr,
}

impl MiniSat {
    pub fn new() -> Self {
        Self::new_custom(CMiniSatFFI::instance())
    }

    pub fn new_custom(ffi: &'static CMiniSatFFI) -> Self {
        unsafe {
            let ptr = ffi.minisat_new();
            ffi.minisat_eliminate(ptr, true);
            MiniSat { ffi, ptr }
        }
    }

    pub fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                self.ffi.minisat_delete(self.ptr);
            }
            self.ptr = std::ptr::null_mut();
        }
    }
}

impl Default for MiniSat {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MiniSat {
    fn drop(&mut self) {
        self.release()
    }
}

impl Debug for MiniSat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiniSat").field("ptr", &self.ptr).finish()
    }
}

impl Display for MiniSat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.signature())
    }
}

/// MiniSat interface.
impl MiniSat {
    pub fn signature(&self) -> &'static str {
        "minisat"
    }

    // Status

    pub fn okay(&self) -> bool {
        unsafe { self.ffi.minisat_okay(self.ptr) }
    }

    // New var/lit

    pub fn new_var(&self) -> Var {
        unsafe { var_from_c(self.ffi.minisat_newVar(self.ptr)) }
    }
    pub fn new_lit(&self) -> Lit {
        unsafe { lit_from_c(self.ffi.minisat_newLit(self.ptr)) }
    }

    // Customize variables

    pub fn set_polarity(&self, var: Var, pol: LBool) {
        unsafe { self.ffi.minisat_setPolarity(self.ptr, var_to_c(var), lbool_to_c(pol, self.ffi)) }
    }
    pub fn set_decision_var(&self, var: Var, pol: bool) {
        unsafe { self.ffi.minisat_setDecisionVar(self.ptr, var_to_c(var), pol.into()) }
    }
    pub fn set_frozen(&self, var: Var, frozen: bool) {
        unsafe { self.ffi.minisat_setFrozen(self.ptr, var_to_c(var), frozen) }
    }

    // Query variable status

    pub fn is_eliminated(&self, var: Var) -> bool {
        unsafe { self.ffi.minisat_isEliminated(self.ptr, var_to_c(var)) }
    }
    pub fn value_var(&self, var: Var) -> LBool {
        unsafe { lbool_from_c(self.ffi.minisat_value_Var(self.ptr, var_to_c(var)), self.ffi) }
    }
    pub fn value_lit(&self, lit: Lit) -> LBool {
        unsafe { lbool_from_c(self.ffi.minisat_value_Lit(self.ptr, lit_to_c(lit)), self.ffi) }
    }

    // Add clause

    pub fn add_clause_begin(&self) {
        unsafe { self.ffi.minisat_addClause_begin(self.ptr) }
    }
    pub fn add_clause_add_lit(&self, lit: Lit) {
        unsafe { self.ffi.minisat_addClause_addLit(self.ptr, lit_to_c(lit)) }
    }
    pub fn add_clause_commit(&self) -> bool {
        unsafe { self.ffi.minisat_addClause_commit(self.ptr) }
    }

    // Simplify

    pub fn simplify(&self) -> bool {
        unsafe { self.ffi.minisat_simplify(self.ptr) }
    }

    // Eliminate

    pub fn eliminate(&self, turn_off_elim: bool) -> bool {
        unsafe { self.ffi.minisat_eliminate(self.ptr, turn_off_elim) }
    }

    // Budget

    pub fn set_conf_budget(&self, x: i32) {
        unsafe { self.ffi.minisat_set_conf_budget(self.ptr, x) }
    }
    pub fn set_prop_budget(&self, x: i32) {
        unsafe { self.ffi.minisat_set_prop_budget(self.ptr, x) }
    }
    pub fn no_budget(&self) {
        unsafe { self.ffi.minisat_no_budget(self.ptr) }
    }

    // Interrupt

    pub fn interrupt(&self) {
        unsafe { self.ffi.minisat_interrupt(self.ptr) }
    }
    pub fn clear_interrupt(&self) {
        unsafe { self.ffi.minisat_clearInterrupt(self.ptr) }
    }

    // Solve

    pub fn solve_begin(&self) {
        unsafe { self.ffi.minisat_solve_begin(self.ptr) }
    }
    pub fn solve_add_lit(&self, lit: Lit) {
        unsafe { self.ffi.minisat_solve_addLit(self.ptr, lit_to_c(lit)) }
    }
    pub fn solve_commit(&self) -> bool {
        unsafe { self.ffi.minisat_solve_commit(self.ptr) }
    }
    pub fn solve_limited_commit(&self) -> LBool {
        unsafe { lbool_from_c(self.ffi.minisat_limited_solve_commit(self.ptr), self.ffi) }
    }

    // Model

    pub fn model_value_var(&self, var: Var) -> LBool {
        unsafe { lbool_from_c(self.ffi.minisat_modelValue_Var(self.ptr, var_to_c(var)), self.ffi) }
    }
    pub fn model_value_lit(&self, lit: Lit) -> LBool {
        unsafe { lbool_from_c(self.ffi.minisat_modelValue_Lit(self.ptr, lit_to_c(lit)), self.ffi) }
    }

    // Statistics

    pub fn num_vars(&self) -> i32 {
        unsafe { self.ffi.minisat_num_vars(self.ptr) }
    }
    pub fn num_clauses(&self) -> i32 {
        unsafe { self.ffi.minisat_num_clauses(self.ptr) }
    }
    pub fn num_assigns(&self) -> i32 {
        unsafe { self.ffi.minisat_num_assigns(self.ptr) }
    }
    pub fn num_free_vars(&self) -> i32 {
        unsafe { self.ffi.minisat_num_freeVars(self.ptr) }
    }
    pub fn num_learnts(&self) -> i32 {
        unsafe { self.ffi.minisat_num_learnts(self.ptr) }
    }
    pub fn num_conflicts(&self) -> i32 {
        unsafe { self.ffi.minisat_num_conflicts(self.ptr) }
    }
    pub fn num_decisions(&self) -> i32 {
        unsafe { self.ffi.minisat_num_decisions(self.ptr) }
    }
    pub fn num_restarts(&self) -> i32 {
        unsafe { self.ffi.minisat_num_restarts(self.ptr) }
    }
    pub fn num_propagations(&self) -> i32 {
        unsafe { self.ffi.minisat_num_propagations(self.ptr) }
    }
}

/// Additional methods.
impl MiniSat {
    pub fn reset(&mut self) {
        self.release();
        self.ptr = unsafe { self.ffi.minisat_new() };
    }

    pub fn set_polarity_lit(&self, lit: Lit, pol: LBool) {
        let var = Var::new(lit.var());
        let pol = match pol {
            LBool::Undef => LBool::Undef,
            _ => {
                if lit.sign() > 0 {
                    // if `lit` is negated (sign=1), flip `pol`
                    pol.flip()
                } else {
                    pol
                }
            }
        };
        self.set_polarity(var, pol);
    }

    pub fn add_clause<I>(&self, lits: I) -> bool
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        self.add_clause_begin();
        let mut max = 0;
        for lit in lits.into_iter().map_into::<Lit>() {
            let var = lit.var() + 1; // 1-based variable index
            max = max.max(var);
            self.add_clause_add_lit(lit);
        }
        // Allocate new variables if necessary
        // for _ in (self.num_vars() + 1)..=max {
        for _ in self.num_vars()..max as _ {
            self.new_var();
        }
        self.add_clause_commit()
    }

    // TODO: remove
    pub fn try_add_clause<I>(&self, lits: I) -> Result<(), <I::Item as TryInto<Lit>>::Error>
    where
        I: IntoIterator,
        I::Item: TryInto<Lit>,
    {
        let lits: Vec<Lit> = lits.into_iter().map(|x| x.try_into()).try_collect()?;
        self.add_clause(lits);
        Ok(())
    }

    pub fn solve_under_assumptions<I>(&self, lits: I) -> bool
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        self.solve_begin();
        for lit in lits.into_iter() {
            self.solve_add_lit(lit.into());
        }
        self.solve_commit()
    }

    pub fn solve(&self) -> bool {
        self.solve_under_assumptions(std::iter::empty::<Lit>())
    }
}
