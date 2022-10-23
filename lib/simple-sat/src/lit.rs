use std::ops;

use crate::var::Var;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Lit(pub(crate) u32);

impl Lit {
    pub const fn new(var: Var, negated: bool) -> Self {
        Lit(var.0 << 1 | negated as u32)
    }

    pub const fn var(self) -> Var {
        Var(self.0 >> 1)
    }

    // TODO: rename to `sign` (with the same same semantics, to match MiniSat)
    pub const fn negated(self) -> bool {
        (self.0 & 1) != 0
    }

    pub const fn sign(self) -> i32 {
        if self.negated() {
            -1
        } else {
            1
        }
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }

    pub const fn to_external(self) -> i32 {
        self.sign() * (self.var().0 + 1) as i32
    }

    pub const fn from_external(lit: i32) -> Lit {
        let var = lit.unsigned_abs() - 1;
        Lit::new(Var(var), lit < 0)
    }
}

// !Lit
impl ops::Not for Lit {
    type Output = Lit;

    fn not(self) -> Self::Output {
        Lit(self.0 ^ 1)
    }
}

// -Lit
impl ops::Neg for Lit {
    type Output = Lit;

    fn neg(self) -> Self::Output {
        Lit(self.0 ^ 1)
    }
}

// Lit ^ bool
impl ops::BitXor<bool> for Lit {
    type Output = Lit;

    fn bitxor(self, rhs: bool) -> Self::Output {
        // Lit::new(self.var(), self.negated() ^ rhs)
        Lit(self.0 ^ rhs as u32)
    }
}
