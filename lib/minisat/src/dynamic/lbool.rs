pub use crate::common::LBool;

use super::ffi::bindings::minisat_lbool;
use super::ffi::CMiniSatFFI;

pub(crate) fn lbool_from_c(lbool: minisat_lbool, ffi: &CMiniSatFFI) -> LBool {
    unsafe {
        if lbool == ffi.minisat_get_l_True() {
            LBool::True
        } else if lbool == ffi.minisat_get_l_False() {
            LBool::False
        } else if lbool == ffi.minisat_get_l_Undef() {
            LBool::Undef
        } else {
            panic!("Bad lbool '{:?}'", lbool)
        }
    }
}

pub(crate) fn lbool_to_c(lbool: LBool, ffi: &CMiniSatFFI) -> minisat_lbool {
    unsafe {
        match lbool {
            LBool::True => ffi.minisat_get_l_True(),
            LBool::False => ffi.minisat_get_l_False(),
            LBool::Undef => ffi.minisat_get_l_Undef(),
        }
    }
}
