use crate::ffi::*;

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
    pub(crate) unsafe fn from_c(lbool: minisat_lbool) -> Self {
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

    pub(crate) unsafe fn to_c(self) -> minisat_lbool {
        match self {
            LBool::True => minisat_l_True,
            LBool::False => minisat_l_False,
            LBool::Undef => minisat_l_Undef,
        }
    }
}
