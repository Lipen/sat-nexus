#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Var(pub(crate) u32);

impl Var {
    pub fn new(var: u32) -> Self {
        Self(var)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}
