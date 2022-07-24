#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
