use crate::ffi::bindings::minisat_Var;

#[derive(Debug, Copy, Clone)]
pub struct Var(minisat_Var);

impl Var {
    pub fn new(var: minisat_Var) -> Self {
        debug_assert!(var.0 >= 0);
        Var(var)
    }
}

impl From<minisat_Var> for Var {
    fn from(var: minisat_Var) -> Self {
        Var(var)
    }
}

// Into<minisat_Var>
impl From<Var> for minisat_Var {
    fn from(x: Var) -> Self {
        x.0
    }
}
