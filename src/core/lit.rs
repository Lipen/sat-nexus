use std::convert::TryInto;
use std::fmt;
use std::ops::Neg;

use crate::ipasir::Lit as IpasirLit;

#[derive(Debug, Copy, Clone)]
pub struct Lit(i32);

impl Lit {
    pub fn new(val: i32) -> Self {
        debug_assert!(val != 0);
        Lit(val)
    }

    pub fn get(self) -> i32 {
        self.0
    }

    pub fn var(self) -> u32 {
        self.get().unsigned_abs()
    }
}

impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<L> From<&L> for Lit
where
    L: Into<Lit> + Copy,
{
    fn from(val: &L) -> Self {
        (*val).into()
    }
}

impl From<i32> for Lit {
    fn from(val: i32) -> Self {
        Self::new(val)
    }
}

impl From<usize> for Lit {
    fn from(val: usize) -> Self {
        Self::new(val.try_into().unwrap())
    }
}

// Into<i32>
impl From<Lit> for i32 {
    fn from(lit: Lit) -> Self {
        lit.0
    }
}

impl From<IpasirLit> for Lit {
    fn from(lit: IpasirLit) -> Self {
        Self::new(lit.into())
    }
}

// Into<IpasirLit>
impl From<Lit> for IpasirLit {
    fn from(lit: Lit) -> Self {
        unsafe { IpasirLit::new_unchecked(lit.0) }
    }
}

impl Neg for Lit {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from(-self.0)
    }
}
