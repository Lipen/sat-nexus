use std::fmt;
use std::ops;

use crate::ffi::bindings::minisat_Lit;
use crate::ffi::bindings::minisat_Var;

use super::Var;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Lit(minisat_Lit);

impl Lit {
    pub fn new(x: minisat_Lit) -> Self {
        debug_assert!(x.0 >= 0);
        Lit(x)
    }

    pub fn var(self) -> Var {
        Var::new(minisat_Var(self.0 .0 >> 1))
    }

    pub fn sign(self) -> i32 {
        self.0 .0 & 1
    }

    pub fn negate(self) -> Self {
        Lit::new(minisat_Lit(self.0 .0 ^ 1))
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
