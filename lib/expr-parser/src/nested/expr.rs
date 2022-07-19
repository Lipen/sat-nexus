use std::fmt::{Display, Formatter};
use std::ops;

use itertools::Itertools;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Const(bool),
    Var(u32),
    Not { arg: Box<Expr> },
    And { args: Vec<Expr> },
    Or { args: Vec<Expr> },
}

// Constructors
impl Expr {
    pub fn not(arg: Self) -> Self {
        Expr::Not { arg: Box::new(arg) }
    }

    pub fn and<I>(args: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        let args = args.into_iter().collect_vec();
        match args.len() {
            // 0 => Expr::Const(true), // 0-ary AND is True
            1 => args.into_iter().next().unwrap(),
            _ => Expr::And { args },
        }
    }

    pub fn or<I>(args: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        let args = args.into_iter().collect_vec();
        match args.len() {
            // 0 => Expr::Const(false), // 0-ary OR is False
            1 => args.into_iter().next().unwrap(),
            _ => Expr::Or { args },
        }
    }
}

impl From<bool> for Expr {
    fn from(b: bool) -> Self {
        Expr::Const(b)
    }
}

impl From<u32> for Expr {
    fn from(value: u32) -> Self {
        Expr::Var(value)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                Expr::Const(b) => {
                    write!(f, "Const({b:#})")
                }
                Expr::Var(var) => {
                    write!(f, "Var({var:#})")
                }
                Expr::Not { arg } => {
                    write!(f, "Not({arg:#})")
                }
                Expr::And { args } => {
                    write!(f, "And({:#})", args.iter().format(", "))
                }
                Expr::Or { args } => {
                    write!(f, "Or({:#})", args.iter().format(", "))
                }
            }
        } else {
            match self {
                Expr::Const(b) => {
                    write!(f, "{b}")
                }
                Expr::Var(var) => {
                    write!(f, "{var}")
                }
                Expr::Not { arg } => {
                    write!(f, "~{arg}")
                }
                Expr::And { args } => {
                    write!(f, "({})", args.iter().format(" & "))
                }
                Expr::Or { args } => {
                    write!(f, "({})", args.iter().format(" | "))
                }
            }
        }
    }
}

impl ops::Not for Expr {
    type Output = Self;

    fn not(self) -> Self::Output {
        Expr::not(self)
    }
}

impl ops::BitAnd for Expr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match self {
            // Auto-consolidate AND
            Expr::And { mut args } => {
                args.push(rhs);
                Expr::and(args)
            }
            e => Expr::And { args: vec![e, rhs] },
        }
    }
}

impl ops::BitOr for Expr {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match self {
            // Auto-consolidate OR
            Expr::Or { mut args } => {
                args.push(rhs);
                Expr::Or { args }
            }
            e => Expr::Or { args: vec![e, rhs] },
        }
    }
}
