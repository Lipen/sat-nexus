use std::fmt;
use std::ops;

use crate::ffi::minisat_Lit;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Lit(minisat_Lit);

impl Lit {
    pub fn new(x: minisat_Lit) -> Self {
        debug_assert!(x >= 0);
        Lit(x)
    }

    /// Returns 0-based variable index.
    pub fn var(self) -> i32 {
        self.0 >> 1
    }

    /// Returns 0 if literal is positive, 1 if negative.
    pub fn sign(self) -> i32 {
        self.0 & 1
    }

    pub fn negate(self) -> Lit {
        (self.0 ^ 1).into()
    }
}

impl From<minisat_Lit> for Lit {
    fn from(lit: minisat_Lit) -> Self {
        Lit(lit)
    }
}

// Into<minisat_Lit>
impl From<Lit> for minisat_Lit {
    fn from(lit: Lit) -> Self {
        lit.0
    }
}

impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
