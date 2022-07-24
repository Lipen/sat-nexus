use crate::common::Var;
use std::fmt::{Display, Formatter};
use std::ops;

/// MiniSat literal.
///
/// **Note:** each literal is represented as `2*var+sign`,
/// where `var` is a variable index (see [Lit::var] and [Var]),
/// and `sign` is a negation bit (see [Lit::sign]).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Lit(u32);

impl Lit {
    pub const fn new(x: u32) -> Self {
        Lit(x)
    }

    pub const fn mk(var: u32, sign: u32) -> Self {
        Self::new(var << 1 | sign)
    }

    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns 0-based variable index.
    pub const fn var(self) -> u32 {
        self.0 >> 1
    }

    /// Returns 0 (false) if literal is positive, 1 (true) if negative.
    pub const fn sign(self) -> u32 {
        self.0 & 1
    }

    pub const fn negate(self) -> Self {
        Self::new(self.0 ^ 1)
    }
}

impl From<Var> for Lit {
    fn from(var: Var) -> Self {
        Self::mk(var.get(), 0)
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ops::Neg for Lit {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.negate()
    }
}

impl ops::Not for Lit {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.negate()
    }
}
