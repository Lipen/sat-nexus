use std::ops;

use crate::var::Var;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Lit(pub(crate) u32);

impl Lit {
    pub fn new(var: Var, negated: bool) -> Self {
        Lit(var.0 << 1 | negated as u32)
    }

    pub fn var(self) -> Var {
        Var(self.0 >> 1)
    }

    // TODO: rename to `sign` (with the same same semantics, to match MiniSat)
    pub fn negated(self) -> bool {
        (self.0 & 1) != 0
    }

    pub fn sign(self) -> i32 {
        if self.negated() {
            -1
        } else {
            1
        }
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn external_lit(self) -> i32 {
        self.sign() * (self.var().0 + 1) as i32
    }

    pub fn from_lit(lit: i32) -> Lit {
        let var = lit.abs() as u32 - 1;
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
