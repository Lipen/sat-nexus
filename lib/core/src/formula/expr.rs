use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops;

use log::debug;
use tap::Tap;

use crate::formula::var::Var;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Const(bool),
    Var(Var),
    Not { arg: Box<Expr> },
    And { lhs: Box<Expr>, rhs: Box<Expr> },
    Or { lhs: Box<Expr>, rhs: Box<Expr> },
}

// Constructors
impl Expr {
    pub fn not(arg: Self) -> Self {
        Expr::Not { arg: Box::new(arg) }
    }

    pub fn and(lhs: Self, rhs: Self) -> Self {
        Expr::And {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    pub fn or(lhs: Self, rhs: Self) -> Self {
        Expr::Or {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
}

impl From<bool> for Expr {
    fn from(b: bool) -> Self {
        Expr::Const(b)
    }
}

impl From<Var> for Expr {
    fn from(var: Var) -> Self {
        Expr::Var(var)
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
                Expr::And { lhs, rhs } => {
                    write!(f, "And({lhs:#}, {rhs:#})")
                }
                Expr::Or { lhs, rhs } => {
                    write!(f, "Or({lhs:#}, {rhs:#})")
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
                Expr::And { lhs, rhs } => {
                    write!(f, "({lhs} & {rhs})")
                }
                Expr::Or { lhs, rhs } => {
                    write!(f, "({lhs} | {rhs})")
                }
            }
        }
    }
}

impl Expr {
    pub fn parse(input: &str) -> eyre::Result<Self> {
        let parsed_expr = expr_parser::parser::parse_expr(input)?;

        use expr_parser::expr::Expr as ParsedExpr;

        fn convert(parsed_expr: ParsedExpr) -> Expr {
            use expr_parser::expr::BinOp;
            match parsed_expr {
                ParsedExpr::Const(b) => Expr::from(b),
                ParsedExpr::Var(v) => Expr::from(Var(v)),
                ParsedExpr::Negation { arg } => Expr::not(convert(*arg)),
                ParsedExpr::BinOp { op, lhs, rhs } => {
                    let lhs = convert(*lhs);
                    let rhs = convert(*rhs);
                    match op {
                        BinOp::And => Expr::and(lhs, rhs),
                        BinOp::Or => Expr::or(lhs, rhs),
                        BinOp::Imply => Expr::or(!lhs, rhs),
                        // TODO: BinOp::Iff and others
                    }
                }
            }
        }

        Ok(convert(parsed_expr))
    }

    pub fn eval(&self, mapping: &HashMap<Var, bool>) -> bool {
        debug!("-> Expr::eval({self})...");
        match self {
            Expr::Const(b) => *b,
            Expr::Var(var) => *mapping.get(var).unwrap_or_else(|| panic!("Mapping does not contain {var}")),
            Expr::Not { arg } => !arg.eval(mapping),
            Expr::And { lhs, rhs } => lhs.eval(mapping) && rhs.eval(mapping),
            Expr::Or { lhs, rhs } => lhs.eval(mapping) || rhs.eval(mapping),
        }
        .tap(|x| debug!("<- Expr::eval({self}) = {x}"))
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
        Expr::and(self, rhs)
    }
}
impl ops::BitAnd<Var> for Expr {
    type Output = Self;

    fn bitand(self, rhs: Var) -> Self::Output {
        Expr::and(self, Expr::from(rhs))
    }
}

impl ops::BitOr for Expr {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Expr::or(self, rhs)
    }
}
impl ops::BitOr<Var> for Expr {
    type Output = Self;

    fn bitor(self, rhs: Var) -> Self::Output {
        Expr::or(self, Expr::from(rhs))
    }
}

#[cfg(test)]
mod tests {
    use log::info;
    use test_log::test;

    use super::*;

    #[test]
    fn test_create_expr() {
        // e1 = x42 & ~True
        let e1 = Expr::And {
            lhs: Box::new(Expr::Var(Var(42))),
            rhs: Box::new(Expr::Not {
                arg: Box::new(Expr::Const(true)),
            }),
        };
        info!("e1 = {:?}", e1);
        info!("e1 = {:#}", e1);
        info!("e1 = {}", e1);

        // e = x42 & ~True
        let e2 = Expr::from(Var(42)) & !Expr::from(true);
        info!("e2 = {:?}", e2);
        info!("e2 = {:#}", e2);
        info!("e2 = {}", e2);

        assert_eq!(e1, e2);
    }

    #[test]
    fn test_eval_expr() {
        // f = x1 & ~False
        let x1 = Var(1);
        let f = x1 & !Expr::from(false);
        info!("f = {:?}", f);
        info!("f = {:#}", f);
        info!("f = {}", f);

        let mut mapping = HashMap::new();

        mapping.insert(x1, true);
        info!("f.eval(mapping={:?}) = {}", mapping, f.eval(&mapping));
        assert_eq!(f.eval(&mapping), true);

        mapping.insert(x1, false);
        info!("f.eval(mapping={:?}) = {}", mapping, f.eval(&mapping));
        assert_eq!(f.eval(&mapping), false);
    }
}
