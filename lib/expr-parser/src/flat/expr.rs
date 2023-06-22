use std::fmt::{Display, Formatter};
use std::ops;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Const(bool),
    Var(u32),
    Negation { arg: Box<Expr> },
    BinOp { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
}

impl Expr {
    pub fn negation(arg: Expr) -> Self {
        Expr::Negation { arg: Box::new(arg) }
    }

    pub fn binop(op: BinOp, lhs: Expr, rhs: Expr) -> Self {
        Expr::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinOp {
    And,
    Or,
    Imply,
    Iff,
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                Expr::Const(b) => {
                    if *b {
                        write!(f, "T")
                    } else {
                        write!(f, "F")
                    }
                }
                Expr::Var(v) => {
                    write!(f, "x{v}")
                }
                Expr::Negation { arg } => {
                    write!(f, "~{arg:#}")
                }
                Expr::BinOp { op, lhs, rhs } => {
                    write!(f, "({lhs:#} {op:#} {rhs:#})")
                }
            }
        } else {
            match self {
                Expr::Const(b) => {
                    write!(f, "{b}")
                }
                Expr::Var(v) => {
                    write!(f, "{v}")
                }
                Expr::Negation { arg } => {
                    write!(f, "~{arg}")
                }
                Expr::BinOp { op, lhs, rhs } => {
                    write!(f, "({lhs} {op} {rhs})")
                }
            }
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if f.alternate() {
                match self {
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    BinOp::Imply => "imply",
                    BinOp::Iff => "iff",
                }
            } else {
                match self {
                    BinOp::And => "&",
                    BinOp::Or => "|",
                    BinOp::Imply => "->",
                    BinOp::Iff => "<=>",
                }
            }
        )
    }
}

impl ops::Not for Expr {
    type Output = Self;

    fn not(self) -> Self::Output {
        Expr::negation(self)
    }
}

impl ops::BitAnd for Expr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Expr::binop(BinOp::And, self, rhs)
    }
}

impl ops::BitOr for Expr {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Expr::binop(BinOp::Or, self, rhs)
    }
}
