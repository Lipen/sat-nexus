use std::fmt;

use itertools::Itertools;

use super::ffi::*;
use super::lbool::*;
use super::lit::*;
use super::var::*;

pub struct MiniSat {
    ptr: *mut minisat_solver,
}

impl MiniSat {
    pub fn new() -> Self {
        let ptr = unsafe { minisat_new() };
        unsafe {
            minisat_eliminate(ptr, true);
        }
        MiniSat { ptr }
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

impl fmt::Display for MiniSat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.signature())
    }
}

/// MiniSat interface.
impl MiniSat {
    pub fn signature(&self) -> &'static str {
        "minisat"
    }

    // Destructor

    pub fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                minisat_delete(self.ptr);
            }
            self.ptr = std::ptr::null_mut();
        }
    }

    // Status

    pub fn okay(&self) -> bool {
        unsafe { minisat_okay(self.ptr) }
    }

    // New var/lit

    pub fn new_var(&self) -> Var {
        unsafe { minisat_newVar(self.ptr) }.into()
    }
    pub fn new_lit(&self) -> Lit {
        unsafe { minisat_newLit(self.ptr) }.into()
    }

    // Customize variables

    pub fn set_polarity(&self, var: Var, pol: LBool) {
        unsafe { minisat_setPolarity(self.ptr, var.into(), pol.to_c()) }
    }
    pub fn set_decision_var(&self, var: Var, pol: bool) {
        unsafe { minisat_setPolarity(self.ptr, var.into(), pol.into()) }
    }
    pub fn set_frozen(&self, var: Var, frozen: bool) {
        unsafe { minisat_setFrozen(self.ptr, var.into(), frozen) }
    }

    // Query variable status

    pub fn is_eliminated(&self, var: Var) -> bool {
        unsafe { minisat_isEliminated(self.ptr, var.into()) }
    }
    pub fn value_var(&self, var: Var) -> LBool {
        unsafe { LBool::from_c(minisat_value_Var(self.ptr, var.into())) }
    }
    pub fn value_lit(&self, lit: Lit) -> LBool {
        unsafe { LBool::from_c(minisat_value_Lit(self.ptr, lit.into())) }
    }

    // Add clause

    pub fn add_clause_begin(&self) {
        unsafe { minisat_addClause_begin(self.ptr) }
    }
    pub fn add_clause_add_lit(&self, lit: Lit) {
        unsafe { minisat_addClause_addLit(self.ptr, lit.into()) }
    }
    pub fn add_clause_commit(&self) -> bool {
        unsafe { minisat_addClause_commit(self.ptr) }
    }

    // Simplify

    pub fn simplify(&self) -> bool {
        unsafe { minisat_simplify(self.ptr) }
    }

    // Eliminate

    pub fn eliminate(&self, turn_off_elim: bool) -> bool {
        unsafe { minisat_eliminate(self.ptr, turn_off_elim) }
    }

    // Budget

    pub fn set_conf_budget(&self, x: i32) {
        unsafe { minisat_set_conf_budget(self.ptr, x) }
    }
    pub fn set_prop_budget(&self, x: i32) {
        unsafe { minisat_set_prop_budget(self.ptr, x) }
    }
    pub fn no_budget(&self) {
        unsafe { minisat_no_budget(self.ptr) }
    }

    // Interrupt

    pub fn interrupt(&self) {
        unsafe { minisat_interrupt(self.ptr) }
    }
    pub fn clear_interrupt(&self) {
        unsafe { minisat_clearInterrupt(self.ptr) }
    }

    // Solve

    pub fn solve_begin(&self) {
        unsafe { minisat_solve_begin(self.ptr) }
    }
    pub fn solve_add_lit(&self, lit: Lit) {
        unsafe { minisat_solve_addLit(self.ptr, lit.into()) }
    }
    pub fn solve_commit(&self) -> bool {
        unsafe { minisat_solve_commit(self.ptr) }
    }
    pub fn solve_limited_commit(&self) -> LBool {
        unsafe { LBool::from_c(minisat_limited_solve_commit(self.ptr)) }
    }

    // Model

    pub fn model_value_var(&self, var: Var) -> LBool {
        unsafe { LBool::from_c(minisat_modelValue_Var(self.ptr, var.into())) }
    }
    pub fn model_value_lit(&self, lit: Lit) -> LBool {
        unsafe { LBool::from_c(minisat_modelValue_Lit(self.ptr, lit.into())) }
    }

    // Statistics

    pub fn num_vars(&self) -> i32 {
        unsafe { minisat_num_vars(self.ptr) }
    }
    pub fn num_clauses(&self) -> i32 {
        unsafe { minisat_num_clauses(self.ptr) }
    }
    pub fn num_assigns(&self) -> i32 {
        unsafe { minisat_num_assigns(self.ptr) }
    }
    pub fn num_free_vars(&self) -> i32 {
        unsafe { minisat_num_freeVars(self.ptr) }
    }
    pub fn num_learnts(&self) -> i32 {
        unsafe { minisat_num_learnts(self.ptr) }
    }
    pub fn num_conflicts(&self) -> i32 {
        unsafe { minisat_num_conflicts(self.ptr) }
    }
    pub fn num_decisions(&self) -> i32 {
        unsafe { minisat_num_decisions(self.ptr) }
    }
    pub fn num_propagations(&self) -> i32 {
        unsafe { minisat_num_propagations(self.ptr) }
    }
    pub fn num_restarts(&self) -> i32 {
        unsafe { minisat_num_restarts(self.ptr) }
    }
}

/// Additional methods.
impl MiniSat {
    pub fn reset(&mut self) {
        self.release();
        self.ptr = unsafe { minisat_new() };
    }

    pub fn set_polarity_lit(&self, lit: Lit, pol: LBool) {
        let var = Var::new(lit.var());
        let pol = match pol {
            LBool::Undef => LBool::Undef,
            _ => {
                if lit.sign() > 0 {
                    pol.flip()
                } else {
                    pol
                }
            }
        };
        self.set_polarity(var, pol);
    }

    pub fn add_clause<I, L>(&self, lits: I) -> bool
    where
        I: IntoIterator<Item = L>,
        L: Into<Lit>,
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
        for _ in self.num_vars()..max {
            self.new_var();
        }
        self.add_clause_commit()
    }

    pub fn try_add_clause<I, L>(&self, lits: I) -> Result<(), <L as TryInto<Lit>>::Error>
    where
        I: IntoIterator<Item = L>,
        L: TryInto<Lit>,
    {
        let lits: Vec<Lit> = lits.into_iter().map(|x| x.try_into()).collect::<Result<_, _>>()?;
        self.add_clause(lits);
        Ok(())
    }

    pub fn solve_under_assumptions<I, L>(&self, lits: I) -> bool
    where
        I: IntoIterator<Item = L>,
        L: Into<Lit>,
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
