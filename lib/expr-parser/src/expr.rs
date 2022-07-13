use std::fmt::{Display, Formatter};
use std::ops;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Const(bool),
    Var(u32),
    Negation { arg: Box<Expr> },
    BinOp { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinOp {
    And,
    Or,
    Imply,
    // Iff,
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Const(b) => {
                write!(f, "{}", b)
            }
            Expr::Var(v) => {
                write!(f, "{}", v)
            }
            Expr::Negation { arg } => {
                write!(f, "~{}", arg)
            }
            Expr::BinOp { op, lhs, rhs } => {
                write!(f, "({} {} {})", lhs, op, rhs)
            }
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinOp::And => "&",
                BinOp::Or => "|",
                BinOp::Imply => "->",
                // BinOp::Iff => "<=>",
            }
        )
    }
}

impl ops::Not for Expr {
    type Output = Self;

    fn not(self) -> Self::Output {
        Expr::Negation { arg: Box::new(self) }
    }
}

impl ops::BitAnd for Expr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Expr::BinOp {
            op: BinOp::And,
            lhs: Box::new(self),
            rhs: Box::new(rhs),
        }
    }
}

impl ops::BitOr for Expr {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Expr::BinOp {
            op: BinOp::Or,
            lhs: Box::new(self),
            rhs: Box::new(rhs),
        }
    }
}
