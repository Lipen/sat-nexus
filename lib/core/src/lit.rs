use std::fmt::{Display, Formatter};
use std::ops::Neg;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Lit(i32);

impl Lit {
    pub const ZERO: Lit = Lit(0);

    pub const fn new(val: i32) -> Self {
        debug_assert!(val != 0, "literal must not be zero, use Lit::ZERO instead");
        Lit(val)
    }

    pub const fn get(self) -> i32 {
        self.0
    }

    pub const fn var(self) -> u32 {
        self.get().unsigned_abs()
    }

    pub const fn sign(self) -> i32 {
        self.get().signum()
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
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

// Into<i32>
impl From<Lit> for i32 {
    fn from(lit: Lit) -> Self {
        lit.get()
    }
}

// -Lit
impl Neg for Lit {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lit_new() {
        let lit = Lit::new(42);
        assert_eq!(lit.get(), 42);
    }

    #[test]
    fn test_lit_display() {
        let lit = Lit::new(42);
        assert_eq!(format!("{}", lit), "42");
    }

    #[test]
    fn test_lit_from_i32() {
        let lit: Lit = 42.into();
        assert_eq!(lit.get(), 42);
    }

    #[test]
    fn test_lit_from_i32_ref() {
        let lit: Lit = (&42).into();
        assert_eq!(lit.get(), 42);
    }

    #[test]
    fn test_lit_into_i32() {
        let lit = Lit::new(42);
        let value: i32 = lit.into();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_lit_neg() {
        let lit = Lit::new(42);
        let neg_lit = -lit;
        assert_eq!(neg_lit.get(), -42);
    }
}
