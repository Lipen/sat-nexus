use std::fmt::{Display, Formatter};

use snafu::Snafu;

use super::Var;

/// A literal of the IPASIR implementing solver.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Lit(i32);

impl Lit {
    /// Creates a new [Lit] from the given value.
    ///
    /// # Safety
    ///
    /// Passed value [val] must be non-zero.
    pub unsafe fn new_unchecked(val: i32) -> Self {
        debug_assert!(val != 0);
        Lit(val)
    }

    /// Returns the backing integer of [self].
    pub fn get(self) -> i32 {
        self.0
    }

    /// Returns the corresponding [Var].
    pub fn var(self) -> Var {
        Var(self.0.unsigned_abs())
    }

    /// Returns the sign.
    pub fn sign(self) -> Sign {
        if self.0.is_positive() {
            Sign::Pos
        } else {
            Sign::Neg
        }
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Into<i32>
impl From<Lit> for i32 {
    fn from(lit: Lit) -> Self {
        lit.0
    }
}

impl From<Var> for Lit {
    fn from(var: Var) -> Self {
        var.lit()
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

#[derive(Debug, Snafu)]
#[snafu(display("Invalid literal value: {}", value))]
pub struct InvalidLitValueError {
    value: i32,
}

impl TryFrom<i32> for Lit {
    type Error = InvalidLitValueError;

    fn try_from(val: i32) -> std::result::Result<Self, Self::Error> {
        if val == 0 || val == i32::MIN {
            return InvalidLitValueSnafu { value: val }.fail();
        }
        Ok(Self(val))
    }
}

impl TryFrom<&i32> for Lit {
    type Error = <Self as TryFrom<i32>>::Error;

    fn try_from(val: &i32) -> std::result::Result<Self, Self::Error> {
        Self::try_from(*val)
    }
}

/// The polarity of a literal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sign {
    /// Positive polarity.
    Pos,
    /// Negative polarity.
    Neg,
}
