pub use crate::common::LBool;

use super::ffi::bindings::minisat_lbool;
use super::ffi::CMiniSatFFI;

pub(crate) fn lbool_from_c(lbool: minisat_lbool, ffi: &CMiniSatFFI) -> LBool {
    if lbool == ffi.minisat_l_true() {
        LBool::True
    } else if lbool == ffi.minisat_l_false() {
        LBool::False
    } else if lbool == ffi.minisat_l_undef() {
        LBool::Undef
    } else {
        panic!("Bad lbool '{:?}'", lbool)
    }
}

pub(crate) fn lbool_to_c(lbool: LBool, ffi: &CMiniSatFFI) -> minisat_lbool {
    match lbool {
        LBool::True => ffi.minisat_l_true(),
        LBool::False => ffi.minisat_l_false(),
        LBool::Undef => ffi.minisat_l_undef(),
    }
}
