use std::ops::{Index, IndexMut};

use crate::idx::VarVec;
use crate::lbool::LBool;
use crate::lit::Lit;
use crate::var::Var;

#[derive(Debug)]
pub struct Assignment {
    pub(crate) assignment: VarVec<LBool>, // {var: value}
}

impl Assignment {
    pub fn new() -> Self {
        Self { assignment: VarVec::new() }
    }
}

// assignment[var]
impl Index<Var> for Assignment {
    type Output = LBool;

    fn index(&self, var: Var) -> &Self::Output {
        self.assignment.index(var)
    }
}

// &mut assignment[var]
impl IndexMut<Var> for Assignment {
    fn index_mut(&mut self, var: Var) -> &mut Self::Output {
        self.assignment.index_mut(var)
    }
}

impl Assignment {
    pub fn value_var(&self, var: Var) -> LBool {
        self[var]
    }
    pub fn value(&self, lit: Lit) -> LBool {
        self.value_var(lit.var()) ^ lit.negated()
    }
}
