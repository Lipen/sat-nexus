use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use std::ops::Neg;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Lit(i32);

impl Lit {
    pub const fn new(val: i32) -> Self {
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

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

impl Neg for Lit {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from(-self.0)
    }
}
