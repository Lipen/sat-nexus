use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Var(pub u32);

impl Var {
    pub const fn new(var: u32) -> Self {
        Self(var)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }

    pub const fn to_external(self) -> u32 {
        self.0 + 1
    }

    pub const fn from_external(var: u32) -> Self {
        Self::new(var - 1)
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_external())
    }
}
