#![allow(dead_code)]

use std::convert::TryFrom;
use std::num::NonZeroI32;

#[derive(Debug, Copy, Clone)]
pub struct Lit(NonZeroI32);

impl Lit {
    pub fn new(val: NonZeroI32) -> Self {
        Lit(val)
    }

    pub unsafe fn new_unchecked(val: i32) -> Self {
        Self::new(NonZeroI32::new_unchecked(val))
    }

    pub fn get(&self) -> i32 {
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
        NonZeroI32::try_from(value).map(|x| Lit(x))
    }
}

impl Into<i32> for Lit {
    fn into(self) -> i32 {
        self.get()
    }
}

// impl From<i32> for Lit {
//     fn from(val: i32) -> Self {
//         debug_assert!(val != 0);
//         Lit(unsafe { NonZeroI32::new_unchecked(val) })
//     }
// }
//
// impl From<&i32> for Lit {
//     fn from(val: &i32) -> Self {
//         Self::from(*val)
//     }
// }
