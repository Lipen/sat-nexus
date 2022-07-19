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

// !Var
impl ops::Not for Var {
    type Output = Expr<Var>;

    fn not(self) -> Self::Output {
        !Expr::from(self)
    }
}

// Var & Var
impl ops::BitAnd for Var {
    type Output = Expr<Var>;

    fn bitand(self, rhs: Var) -> Self::Output {
        Expr::from(self) & Expr::from(rhs)
    }
}
// Var & Expr
impl ops::BitAnd<Expr<Var>> for Var {
    type Output = Expr<Var>;

    fn bitand(self, rhs: Expr<Var>) -> Self::Output {
        Expr::from(self) & rhs
    }
}

// Var | Var
impl ops::BitOr for Var {
    type Output = Expr<Var>;

    fn bitor(self, rhs: Var) -> Self::Output {
        Expr::from(self) | Expr::from(rhs)
    }
}
// Var | Expr
impl ops::BitOr<Expr<Var>> for Var {
    type Output = Expr<Var>;

    fn bitor(self, rhs: Expr<Var>) -> Self::Output {
        Expr::from(self) | rhs
    }
}
