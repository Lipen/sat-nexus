pub use crate::common::Var;

use super::ffi::minisat_Var;

pub(crate) fn var_from_c(var: minisat_Var) -> Var {
    Var::new(var as _)
}

pub(crate) fn var_to_c(var: Var) -> minisat_Var {
    var.get() as _
}
