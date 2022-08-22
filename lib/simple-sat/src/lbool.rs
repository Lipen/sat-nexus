use std::ops::BitXor;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum LBool {
    False = 0,
    True = 1,
    Undef = 2,
}

impl LBool {
    #[inline]
    pub fn is_undef(self) -> bool {
        (self as u8) > 1
    }
}

impl From<bool> for LBool {
    fn from(b: bool) -> Self {
        if b {
            LBool::True
        } else {
            LBool::False
        }
    }
}

// LBool ^ bool
impl BitXor<bool> for LBool {
    type Output = LBool;

    fn bitxor(self, rhs: bool) -> Self::Output {
        match self {
            LBool::False => {
                if rhs {
                    LBool::True
                } else {
                    LBool::False
                }
            }
            LBool::True => {
                if rhs {
                    LBool::False
                } else {
                    LBool::True
                }
            }
            LBool::Undef => LBool::Undef,
        }
    }
}
