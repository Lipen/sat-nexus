use std::fmt;

use tap::Pipe;

use super::ffi::*;
use super::types::*;

pub struct MiniSat {
    ffi: &'static MiniSatFFI,
    ptr: MiniSatPtr,
}

impl MiniSat {
    pub fn new() -> Self {
        Self::new_custom(MiniSatFFI::instance())
    }

    pub fn new_custom(ffi: &'static MiniSatFFI) -> Self {
        let ptr = ffi.init();
        unsafe {
            ffi.minisat_eliminate(ptr, true);
        }
        MiniSat { ffi, ptr }
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
                self.ffi.minisat_delete(self.ptr);
            }
            self.ptr = std::ptr::null_mut();
        }
    }

    // Status

    pub fn okay(&self) -> bool {
        unsafe { self.ffi.minisat_okay(self.ptr) }
    }

    // New var/lit

    pub fn new_var(&self) -> Var {
        unsafe { self.ffi.minisat_newVar(self.ptr) }.into()
    }
    pub fn new_lit(&self) -> Lit {
        unsafe { self.ffi.minisat_newLit(self.ptr) }.into()
    }

    // Customize variables

    pub fn set_polarity(&self, var: Var, pol: LBool) {
        unsafe { self.ffi.minisat_setPolarity(self.ptr, var.into(), pol.to_c(self.ffi)) }
    }
    pub fn set_decision_var(&self, var: Var, pol: bool) {
        unsafe { self.ffi.minisat_setPolarity(self.ptr, var.into(), pol.into()) }
    }
    pub fn set_frozen(&self, var: Var, frozen: bool) {
        unsafe { self.ffi.minisat_setFrozen(self.ptr, var.into(), frozen) }
    }

    // Query variable status

    pub fn is_eliminated(&self, var: Var) -> bool {
        unsafe { self.ffi.minisat_isEliminated(self.ptr, var.into()) }
    }
    pub fn value_var(&self, var: Var) -> LBool {
        unsafe { self.ffi.minisat_value_Var(self.ptr, var.into()) }.pipe(|x| self.lbool(x))
    }
    pub fn value_lit(&self, lit: Lit) -> LBool {
        unsafe { self.ffi.minisat_value_Lit(self.ptr, lit.into()) }.pipe(|x| self.lbool(x))
    }

    // Add clause

    pub fn add_clause_begin(&self) {
        unsafe { self.ffi.minisat_addClause_begin(self.ptr) }
    }
    pub fn add_clause_add_lit(&self, lit: Lit) {
        unsafe { self.ffi.minisat_addClause_addLit(self.ptr, lit.into()) }
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
        unsafe { self.ffi.minisat_solve_addLit(self.ptr, lit.into()) }
    }
    pub fn solve_commit(&self) -> bool {
        unsafe { self.ffi.minisat_solve_commit(self.ptr) }
    }
    pub fn solve_limited_commit(&self) -> LBool {
        unsafe { self.ffi.minisat_limited_solve_commit(self.ptr) }.pipe(|x| self.lbool(x))
    }

    // Model

    pub fn model_value_var(&self, var: Var) -> LBool {
        unsafe { self.ffi.minisat_modelValue_Var(self.ptr, var.into()) }.pipe(|x| self.lbool(x))
    }
    pub fn model_value_lit(&self, lit: Lit) -> LBool {
        unsafe { self.ffi.minisat_modelValue_Lit(self.ptr, lit.into()) }.pipe(|x| self.lbool(x))
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
    pub fn num_propagations(&self) -> i32 {
        unsafe { self.ffi.minisat_num_propagations(self.ptr) }
    }
    pub fn num_restarts(&self) -> i32 {
        unsafe { self.ffi.minisat_num_restarts(self.ptr) }
    }

    // Extra methods

    // TODO:
    //
    // int             minisat_conflict_len    (minisat_solver *s);
    // minisat_Lit     minisat_conflict_nthLit (minisat_solver *s, int i);
    //
    // void            minisat_set_conf_budget (minisat_solver* s, int x);
    // void            minisat_set_prop_budget (minisat_solver* s, int x);
    // void            minisat_no_budget       (minisat_solver* s);
}

/// Additional methods.
impl MiniSat {
    //noinspection RsSelfConvention
    fn lbool(&self, lbool: bindings::minisat_lbool) -> LBool {
        LBool::from_c(self.ffi, lbool)
    }

    pub fn reset(&mut self) {
        self.release();
        self.ptr = self.ffi.init();
    }

    pub fn set_polarity_lit(&self, lit: Lit, pol: LBool) {
        let var = lit.var();
        let pol = match pol {
            LBool::Undef => pol,
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
        for lit in lits.into_iter() {
            self.add_clause_add_lit(lit.into());
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
