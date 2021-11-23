use std::convert::TryFrom;
use std::fmt;

use snafu::Snafu;

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

impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            return InvalidLitValueContext { value: val }.fail();
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

/// A variable of the IPASIR implementing solver.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Var(pub(crate) u32);

impl Var {
    fn lit(&self) -> Lit {
        unsafe { Lit::new_unchecked(self.0 as i32) }
    }
}

impl From<Lit> for Var {
    fn from(lit: Lit) -> Self {
        lit.var()
    }
}

/// A clause from the IPASIR solver.
///
/// Note: last literal is 0.
pub struct Clause<'a> {
    /// The zero-ended literals.
    lits: &'a [Lit],
}

impl<'a> Clause<'a> {
    /// Returns the length of the clause.
    pub fn len(&self) -> usize {
        self.lits.len()
    }

    /// Returns `true` if the clause is empty.
    ///
    /// # Note
    ///
    /// Normally a clause should never be empty.
    pub fn is_empty(&self) -> bool {
        self.lits.len() == 0
    }

    /// Returns an iterator over the literals of the clause.
    pub fn iter(&self) -> impl Iterator<Item = &Lit> {
        self.lits.iter()
    }
}

impl<'a> From<&'a [Lit]> for Clause<'a> {
    fn from(lits: &'a [Lit]) -> Self {
        debug_assert!(!lits.is_empty());
        debug_assert_eq!(lits.last(), Some(&Lit(0)));
        Self { lits }
    }
}

impl<'a, Idx> std::ops::Index<Idx> for Clause<'a>
where
    Idx: std::slice::SliceIndex<[Lit]>,
{
    type Output = <[Lit] as std::ops::Index<Idx>>::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.lits[index]
    }
}
