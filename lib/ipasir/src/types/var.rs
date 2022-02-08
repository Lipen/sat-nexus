use super::Lit;

/// A variable of the IPASIR implementing solver.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Var(pub(crate) u32);

impl Var {
    pub(crate) fn lit(&self) -> Lit {
        unsafe { Lit::new_unchecked(self.0 as i32) }
    }
}

impl From<Lit> for Var {
    fn from(lit: Lit) -> Self {
        lit.var()
    }
}
