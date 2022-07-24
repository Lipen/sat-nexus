pub use crate::common::LBool;

use super::ffi::*;

pub(crate) unsafe fn lbool_from_c(lbool: minisat_lbool) -> LBool {
    if lbool == minisat_l_True {
        LBool::True
    } else if lbool == minisat_l_False {
        LBool::False
    } else if lbool == minisat_l_Undef {
        LBool::Undef
    } else {
        panic!("Bad lbool '{:?}'", lbool)
    }
}

pub(crate) unsafe fn lbool_to_c(lbool: LBool) -> minisat_lbool {
    match lbool {
        LBool::True => minisat_l_True,
        LBool::False => minisat_l_False,
        LBool::Undef => minisat_l_Undef,
    }
}
