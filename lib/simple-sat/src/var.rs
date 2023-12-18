use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Var(u32);

impl Var {
    pub const fn new(var: u32) -> Self {
        Self(var)
    }

    pub const fn inner(self) -> u32 {
        self.0
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }

    pub fn to_external(self) -> u32 {
        self.0 + 1
    }

    pub fn from_external(var: u32) -> Self {
        assert_ne!(var, 0);
        Self::new(var - 1)
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_external())
    }
}
