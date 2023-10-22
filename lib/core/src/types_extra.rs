#![allow(dead_code)]

use std::convert::TryFrom;
use std::num::NonZeroI32;

#[derive(Debug, Copy, Clone)]
pub struct Lit(NonZeroI32);

impl Lit {
    pub const fn new(val: NonZeroI32) -> Self {
        Lit(val)
    }

    pub const unsafe fn new_unchecked(val: i32) -> Self {
        Self::new(NonZeroI32::new_unchecked(val))
    }

    pub const fn get(&self) -> i32 {
        self.0.get()
    }
}

impl From<NonZeroI32> for Lit {
    fn from(val: NonZeroI32) -> Self {
        Self::new(val)
    }
}

impl TryFrom<i32> for Lit {
    type Error = <NonZeroI32 as TryFrom<i32>>::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        NonZeroI32::try_from(value).map(Lit)
    }
}

// Into<i32>
impl From<Lit> for i32 {
    fn from(lit: Lit) -> Self {
        lit.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lit_new() {
        let val = NonZeroI32::new(42).unwrap();
        let lit = Lit::new(val);

        assert_eq!(lit.get(), 42);
    }

    #[test]
    fn test_lit_new_unchecked() {
        let lit = unsafe { Lit::new_unchecked(42) };

        assert_eq!(lit.get(), 42);
    }

    #[test]
    fn test_lit_from_non_zero_i32() {
        let val = NonZeroI32::new(42).unwrap();
        let lit: Lit = val.into();

        assert_eq!(lit.get(), 42);
    }

    #[test]
    fn test_lit_try_from_i32() {
        let lit: Result<Lit, _> = 42.try_into();

        assert!(lit.is_ok());
        assert_eq!(lit.unwrap().get(), 42);

        let zero_lit: Result<Lit, _> = 0.try_into();
        assert!(zero_lit.is_err());
    }

    #[test]
    fn test_lit_into_i32() {
        let lit = Lit::new(NonZeroI32::new(42).unwrap());
        let value: i32 = lit.into();

        assert_eq!(value, 42);
    }
}
