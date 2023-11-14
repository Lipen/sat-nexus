use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Var(pub(crate) u32);

impl Var {
    pub const fn new(var: u32) -> Self {
        Self(var)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // display Var as 1-based integer:
        write!(f, "{}", self.index() + 1)
    }
}
