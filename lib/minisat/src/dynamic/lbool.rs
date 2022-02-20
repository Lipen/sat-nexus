use super::ffi::bindings::minisat_lbool;
use super::ffi::MiniSatFFI;

#[derive(Debug, Copy, Clone)]
pub enum LBool {
    True,  // = 1,
    False, // = 0,
    Undef, // = -1,
}

impl LBool {
    pub fn bool(self) -> bool {
        match self {
            LBool::True => true,
            LBool::False => false,
            LBool::Undef => panic!("LBool::Undef cannot be safely converted to bool"),
        }
    }

    pub fn flip(self) -> Self {
        match self {
            LBool::True => LBool::False,
            LBool::False => LBool::True,
            LBool::Undef => panic!("LBool::Undef cannot be safely flipped"),
        }
    }
}

impl LBool {
    pub(crate) fn from_c(ffi: &MiniSatFFI, lbool: minisat_lbool) -> Self {
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

    pub(crate) fn to_c(self, ffi: &MiniSatFFI) -> minisat_lbool {
        match self {
            LBool::True => ffi.minisat_l_true(),
            LBool::False => ffi.minisat_l_false(),
            LBool::Undef => ffi.minisat_l_undef(),
        }
    }
}
