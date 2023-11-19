use tracing::debug;

use crate::cref::ClauseRef;
use crate::idx::VarVec;
use crate::lbool::LBool;
use crate::lit::Lit;
use crate::var::Var;

#[derive(Debug)]
pub struct VarData {
    pub(crate) reason: Option<ClauseRef>,
    pub(crate) level: usize,
}

#[derive(Debug)]
pub struct Assignment {
    pub(crate) assignment: VarVec<LBool>, // {var: value}
    pub(crate) var_data: VarVec<VarData>, // {var: {reason,level}}
    pub(crate) trail: Vec<Lit>,
    pub(crate) trail_lim: Vec<usize>,
    pub(crate) qhead: usize,
}

impl Assignment {
    pub const fn new() -> Self {
        Self {
            assignment: VarVec::new(),
            var_data: VarVec::new(),
            trail: Vec::new(),
            trail_lim: Vec::new(),
            qhead: 0,
        }
    }

    pub fn value(&self, lit: Lit) -> LBool {
        self.assignment[lit.var()] ^ lit.negated()
    }
    pub fn value_var(&self, var: Var) -> LBool {
        self.assignment[var]
    }
    pub fn assign(&mut self, var: Var, value: LBool) {
        self.assignment[var] = value;
    }
    pub fn unassign(&mut self, var: Var) {
        self.assignment[var] = LBool::Undef;
    }

    pub fn var_data(&self, var: Var) -> &VarData {
        &self.var_data[var]
    }
    pub fn reason(&self, var: Var) -> Option<ClauseRef> {
        self.var_data[var].reason
    }
    pub fn level(&self, var: Var) -> usize {
        self.var_data[var].level
    }

    pub fn fixed(&self, lit: Lit) -> LBool {
        if self.level(lit.var()) > 0 {
            LBool::Undef
        } else {
            self.value(lit)
        }
    }
    pub fn fixed_var(&self, var: Var) -> LBool {
        if self.level(var) > 0 {
            LBool::Undef
        } else {
            self.value_var(var)
        }
    }

    pub fn decision_level(&self) -> usize {
        self.trail_lim.len()
    }
    pub fn new_decision_level(&mut self) {
        self.trail_lim.push(self.trail.len());
    }

    pub fn unchecked_enqueue(&mut self, lit: Lit, reason: Option<ClauseRef>) {
        debug_assert_eq!(self.value(lit), LBool::Undef);

        self.assignment[lit.var()] = LBool::from(!lit.negated());
        self.var_data[lit.var()] = VarData {
            reason,
            level: self.decision_level(),
        };
        self.trail.push(lit);
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
    pub fn enqueue(&mut self, lit: Lit, reason: Option<ClauseRef>) -> bool {
        match self.value(lit) {
            LBool::Undef => {
                self.unchecked_enqueue(lit, reason);
                true
            }
            LBool::True => {
                debug!("existing consistent assignment of {:?}", lit);
                true
            }
            LBool::False => {
                // conflict
                false
            }
        }
    }

    pub fn dequeue(&mut self) -> Option<Lit> {
        if self.qhead < self.trail.len() {
            let p = self.trail[self.qhead];
            self.qhead += 1;
            Some(p)
        } else {
            None
        }
    }
}
