use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops;

use itertools::{all, any, Itertools};

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
        // Double negation: Not(Not(x)) == x
        match arg {
            Expr::Not { arg: sub_arg } => *sub_arg,
            _ => Expr::Not { arg: Box::new(arg) },
        }
    }

    pub fn and<I>(args: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Self>,
    {
        // Auto-consolidate: AND(x1,AND(x2,x3)) == AND(x1,x2,x3)
        let mut new_args = Vec::new();
        for arg in args.into_iter().map_into::<Self>() {
            match arg {
                Expr::And { args: sub_args } => {
                    new_args.extend(sub_args);
                }
                _ => new_args.push(arg),
            }
        }
        match new_args.len() {
            // 0 => Expr::Const(true), // 0-ary AND is True
            1 => new_args.into_iter().next().unwrap(), // single arg
            _ => Expr::And { args: new_args },
        }
    }

    pub fn or<I>(args: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Self>,
    {
        // Auto-consolidate: OR(x1,OR(x2,x3)) == OR(x1,x2,x3)
        let mut new_args = Vec::new();
        for arg in args.into_iter().map_into::<Self>() {
            match arg {
                Expr::Or { args: sub_args } => {
                    new_args.extend(sub_args);
                }
                _ => new_args.push(arg),
            }
        }
        match new_args.len() {
            // 0 => Expr::Const(false), // 0-ary OR is False
            1 => new_args.into_iter().next().unwrap(), // single arg
            _ => Expr::Or { args: new_args },
        }
    }

    pub fn imply(lhs: impl Into<Self>, rhs: impl Into<Self>) -> Self {
        !lhs.into() | rhs.into()
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
        Expr::and([self, rhs])
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
        Expr::or([self, rhs])
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
    pub fn parse_flat(input: &str) -> eyre::Result<Self> {
        let parsed_expr = expr_parser::flat::parser::parse_expr(input)?;

        use expr_parser::flat::expr::BinOp;
        use expr_parser::flat::expr::Expr as ParsedExpr;

        fn convert(parsed_expr: ParsedExpr) -> Expr<Var> {
            match parsed_expr {
                ParsedExpr::Const(b) => Expr::Const(b),
                ParsedExpr::Var(v) => Expr::Terminal(Var(v)),
                ParsedExpr::Negation { arg } => Expr::not(convert(*arg)),
                ParsedExpr::BinOp { op, lhs, rhs } => {
                    let lhs = convert(*lhs);
                    let rhs = convert(*rhs);
                    match op {
                        BinOp::And => lhs & rhs,
                        BinOp::Or => lhs | rhs,
                        BinOp::Imply => Expr::imply(lhs, rhs),
                        // other ops
                    }
                }
            }
        }

        Ok(convert(parsed_expr))
    }

    pub fn parse_nested(input: &str) -> eyre::Result<Self> {
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
