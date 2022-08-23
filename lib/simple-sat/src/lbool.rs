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
    pub const fn is_undef(self) -> bool {
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
        // https://godbolt.org/z/ffKKW7vTx (see bitxor6)
        match self {
            LBool::Undef => LBool::Undef,

            // SAFETY: both lhs (`self as u8`) and rhs (`rhs as u8`) are 0/1,
            // so their xor is also 0/1, which is safe to transmute into LBool.
            _ => unsafe { std::mem::transmute((self as u8) ^ (rhs as u8)) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lbool_bitxor() {
        assert_eq!(LBool::False ^ false, LBool::False);
        assert_eq!(LBool::False ^ true, LBool::True);
        assert_eq!(LBool::True ^ false, LBool::True);
        assert_eq!(LBool::True ^ true, LBool::False);
        assert_eq!(LBool::Undef ^ false, LBool::Undef);
        assert_eq!(LBool::Undef ^ true, LBool::Undef);
    }
}
