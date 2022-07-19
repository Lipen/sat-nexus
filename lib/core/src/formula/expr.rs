use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::iter::once;
use std::ops;

use itertools::{all, any, chain, Itertools};

use crate::formula::nnf::NNF;
use crate::formula::var::Var;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<T> {
    Const(bool),
    Terminal(T),
    Not { arg: Box<Expr<T>> },
    And { args: Vec<Expr<T>> },
    Or { args: Vec<Expr<T>> },
}

// Constructors
impl<T> Expr<T> {
    pub fn not(arg: Self) -> Self {
        Expr::Not { arg: Box::new(arg) }
    }

    pub fn and<I>(args: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Self>,
    {
        let args = args.into_iter().map_into::<Self>().collect_vec();
        match args.len() {
            // 0 => Expr::Const(true), // 0-ary AND is True
            1 => args.into_iter().next().unwrap(),
            _ => Expr::And { args },
        }
    }

    pub fn or<I>(args: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Self>,
    {
        let args = args.into_iter().map_into::<Self>().collect_vec();
        match args.len() {
            // 0 => Expr::Const(false), // 0-ary OR is False
            1 => args.into_iter().next().unwrap(),
            _ => Expr::Or { args },
        }
    }
}

impl<T> From<T> for Expr<T> {
    fn from(value: T) -> Self {
        Expr::Terminal(value)
    }
}

impl<T> Display for Expr<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                Expr::Const(b) => {
                    write!(f, "Const({b:#})")
                }
                Expr::Terminal(value) => {
                    write!(f, "Terminal({value:#})")
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
                Expr::Terminal(value) => {
                    write!(f, "{value}")
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

// !Expr
impl<T> ops::Not for Expr<T> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Expr::not(self)
    }
}

// Expr & Expr
impl<T> ops::BitAnd for Expr<T> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        // Auto-consolidate
        use Expr::And;
        match (self, rhs) {
            (And { args: lhs_args }, And { args: rhs_args }) => Expr::and(chain(lhs_args, rhs_args)),
            (And { args: lhs_args }, rhs) => Expr::and(chain(lhs_args, once(rhs))),
            (lhs, And { args: rhs_args }) => Expr::and(chain(once(lhs), rhs_args)),
            (lhs, rhs) => Expr::and([lhs, rhs]),
        }
    }
}
// Expr & Var
impl ops::BitAnd<Var> for Expr<Var> {
    type Output = Self;

    fn bitand(self, rhs: Var) -> Self::Output {
        self & Expr::from(rhs)
    }
}

// Expr | Expr
impl<T> ops::BitOr for Expr<T> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        // Auto-consolidate
        use Expr::Or;
        match (self, rhs) {
            (Or { args: lhs_args }, Or { args: rhs_args }) => Expr::or(chain!(lhs_args, rhs_args)),
            (Or { args: lhs_args }, rhs) => Expr::or(chain(lhs_args, once(rhs))),
            (lhs, Or { args: rhs_args }) => Expr::or(chain(once(lhs), rhs_args)),
            (lhs, rhs) => Expr::or([lhs, rhs]),
        }
    }
}
// Expr | Var
impl ops::BitOr<Var> for Expr<Var> {
    type Output = Self;

    fn bitor(self, rhs: Var) -> Self::Output {
        self | Expr::from(rhs)
    }
}

impl<T> Expr<T> {
    pub fn eval(&self, mapping: &HashMap<T, bool>) -> bool
    where
        T: Hash + Eq + Debug,
    {
        match self {
            Expr::Const(b) => *b,
            Expr::Terminal(v) => *mapping.get(v).unwrap_or_else(|| panic!("Mapping does not contain {v:?}")),
            Expr::Not { arg } => !arg.eval(mapping),
            Expr::And { args } => all(args, |arg| arg.eval(mapping)),
            Expr::Or { args } => any(args, |arg| arg.eval(mapping)),
        }
    }
}

impl Expr<Var> {
    pub fn parse(input: &str) -> eyre::Result<Self> {
        let parsed_expr = expr_parser::nested::parser::parse_expr(input)?;

        use expr_parser::nested::expr::Expr as ParsedExpr;

        fn convert(parsed_expr: ParsedExpr) -> Expr<Var> {
            match parsed_expr {
                ParsedExpr::Const(b) => Expr::Const(b),
                ParsedExpr::Var(v) => Expr::Terminal(Var(v)),
                ParsedExpr::Not { arg } => Expr::not(convert(*arg)),
                ParsedExpr::And { args } => Expr::and(args.into_iter().map(convert)),
                ParsedExpr::Or { args } => Expr::or(args.into_iter().map(convert)),
            }
        }

        Ok(convert(parsed_expr))
    }
}

// ==========================================

pub trait Terminal {
    // Must return NNF::Literal
    fn to_nnf(&self) -> NNF;

    fn negated_to_nnf(&self) -> NNF {
        match self.to_nnf() {
            NNF::Literal(lit) => NNF::Literal(-lit),
            _ => panic!("Terminal::to_nnf() returned not Literal"),
        }
    }
}

impl Terminal for Var {
    fn to_nnf(&self) -> NNF {
        NNF::Literal(self.0 as i32)
    }
}

// NNF conversion
impl<T> Expr<T>
where
    T: Terminal,
{
    pub fn to_nnf(&self) -> NNF {
        match self {
            Expr::Const(b) => Self::constant_to_nnf(*b),
            Expr::Terminal(value) => Self::terminal_to_nnf(value),
            Expr::Not { arg } => Self::negation_to_nnf(arg),
            Expr::And { args } => Self::conjunction_to_nnf(args),
            Expr::Or { args } => Self::disjunction_to_nnf(args),
        }
    }

    fn constant_to_nnf(_b: bool) -> NNF {
        panic!("Constants are not supported")
    }

    /// Converts the terminal [`value`] to [`NNF`].
    fn terminal_to_nnf(value: &T) -> NNF {
        value.to_nnf()
    }

    /// Converts the negation of [`arg`] to [`NNF`].
    fn negation_to_nnf(arg: &Expr<T>) -> NNF {
        match arg {
            Expr::Const(_) => panic!("Negation of constants are not supported"),
            Expr::Terminal(value) => value.negated_to_nnf(),
            Expr::Not { arg } => arg.to_nnf(),
            Expr::And { args } => NNF::or(args.iter().map(|arg| Self::negation_to_nnf(arg))),
            Expr::Or { args } => NNF::and(args.iter().map(|arg| Self::negation_to_nnf(arg))),
        }
    }

    /// Converts the conjunction of [`args`] to [`NNF`].
    fn conjunction_to_nnf(args: &[Expr<T>]) -> NNF {
        NNF::and(args.iter().map(|arg| arg.to_nnf()))
    }

    /// Converts the disjunction of [`args`] to [`NNF`].
    fn disjunction_to_nnf(args: &[Expr<T>]) -> NNF {
        NNF::or(args.iter().map(|arg| arg.to_nnf()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_to_nnf() {
        let e: Expr<Var> = Expr::And {
            args: vec![
                Expr::Not {
                    arg: Box::new(Expr::Not {
                        arg: Box::new(Expr::Terminal(Var(8))),
                    }),
                },
                Expr::Not {
                    arg: Box::new(Expr::Or {
                        args: vec![Expr::Terminal(Var(1)), Expr::Terminal(Var(2))],
                    }),
                },
            ],
        };
        println!("e = {:?}", e);
        let nnf = e.to_nnf();
        println!("nnf = {:?}", nnf);
    }
}
