use std::fmt::{Display, Formatter};
use std::ops;

use crate::formula::expr::Expr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Var(pub u32);

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "x{}", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl ops::Not for Var {
    type Output = Expr;

    fn not(self) -> Self::Output {
        !Expr::from(self)
    }
}

impl ops::BitAnd for Var {
    type Output = Expr;

    fn bitand(self, rhs: Var) -> Self::Output {
        Expr::from(self) & Expr::from(rhs)
    }
}
impl ops::BitAnd<Expr> for Var {
    type Output = Expr;

    fn bitand(self, rhs: Expr) -> Self::Output {
        Expr::from(self) & rhs
    }
}

impl ops::BitOr for Var {
    type Output = Expr;

    fn bitor(self, rhs: Var) -> Self::Output {
        Expr::from(self) | Expr::from(rhs)
    }
}
impl ops::BitOr<Expr> for Var {
    type Output = Expr;

    fn bitor(self, rhs: Expr) -> Self::Output {
        Expr::from(self) | rhs
    }
}
