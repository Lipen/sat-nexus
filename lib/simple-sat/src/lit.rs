use std::fmt::{Display, Formatter};
use std::ops;

use crate::var::Var;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Lit(u32);

impl Lit {
    pub const fn new(var: Var, negated: bool) -> Self {
        Lit(var.inner() << 1 | negated as u32)
    }
    pub const fn positive(var: Var) -> Self {
        Self::new(var, false)
    }
    pub const fn negative(var: Var) -> Self {
        Self::new(var, true)
    }

    pub const fn inner(self) -> u32 {
        self.0
    }

    pub const fn var(self) -> Var {
        Var::new(self.0 >> 1)
    }

    // TODO: rename to `sign` (with the same same semantics, to match MiniSat)
    pub const fn negated(self) -> bool {
        (self.0 & 1) != 0
    }

    pub const fn to_external(self) -> i32 {
        let x = self.var().to_external() as i32;
        if self.negated() {
            -x
        } else {
            x
        }
    }

    //noinspection RsAssertEqual (const_assert)
    pub const fn from_external(lit: i32) -> Self {
        assert!(lit != 0, "External literal cannot be zero");
        let var = Var::from_external(lit.unsigned_abs());
        let negated = lit < 0;
        Self::new(var, negated)
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

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_external())
    }
}
