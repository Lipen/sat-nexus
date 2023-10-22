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
