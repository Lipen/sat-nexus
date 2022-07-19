use log::debug;
use once_cell::sync::Lazy;
use pest::error::Error;
use pest::iterators::Pair;
use pest::prec_climber::PrecClimber;
use pest::Parser;

use super::expr::{BinOp, Expr};

#[derive(Parser)]
#[grammar = "grammar/flat.pest"] // relative to project `src`
struct ExprParser;

static PREC_CLIMBER: Lazy<PrecClimber<Rule>> = Lazy::new(|| {
    use pest::prec_climber::{Assoc::*, Operator};
    use Rule::*;

    // Precedence is defined lowest to highest
    PrecClimber::new(vec![
        // Operator::new(iff, Left),
        Operator::new(imply, Right),
        Operator::new(or, Left),
        Operator::new(and, Left),
    ])
});

pub fn parse_expr(input: &str) -> Result<Expr, Error<Rule>> {
    let expr = ExprParser::parse(Rule::main, input)?.next().unwrap();

    fn parse_expr(expr: Pair<Rule>) -> Expr {
        debug!("expr = {:?} = {}", expr.as_str(), expr);
        let infix = |lhs: Expr, op: Pair<Rule>, rhs: Expr| {
            debug!("op = {:?} = {}", op.as_str(), op);
            let op = match op.as_rule() {
                Rule::and => BinOp::And,
                Rule::or => BinOp::Or,
                Rule::imply => BinOp::Imply,
                // Rule::iff => BinOp::Iff,
                _ => unreachable!(),
            };
            Expr::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        };
        PREC_CLIMBER.climb(expr.into_inner(), parse_atom, infix)
    }

    fn parse_atom(atom: Pair<Rule>) -> Expr {
        debug!("atom = {:?} = {}", atom.as_str(), atom);
        match atom.as_rule() {
            Rule::expr => {
                // Braced expression
                parse_expr(atom)
            }
            Rule::negated_atom => {
                let a = atom.into_inner().next().unwrap();
                debug!("a = {:?} = {}", a.as_str(), a);
                Expr::Negation {
                    arg: Box::new(parse_atom(a)),
                }
            }
            Rule::variable => {
                assert_eq!(&atom.as_str()[..1], "x");
                let v: u32 = atom.as_str()[1..].parse().unwrap();
                debug!("v = {}", v);
                Expr::Var(v)
            }
            Rule::bool => {
                let b = atom.into_inner().next().unwrap();
                debug!("b = {:?} = {}", b.as_str(), b);
                match b.as_rule() {
                    Rule::true_lit => Expr::Const(true),
                    Rule::false_lit => Expr::Const(false),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(parse_expr(expr))
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    const X1: Expr = Expr::Var(1);
    const X2: Expr = Expr::Var(2);
    const X3: Expr = Expr::Var(3);
    const TRUE: Expr = Expr::Const(true);
    const FALSE: Expr = Expr::Const(false);

    #[test]
    fn test_single_var() {
        let s = "x1";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(X1));
    }

    #[test]
    fn test_braced_single_var() {
        let s = "(x1)";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(X1));
    }
    #[test]
    fn test_double_braced_single_var() {
        let s = "((x1))";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(X1));
    }

    #[test]
    fn test_negative_single_var() {
        let s = "~x1";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(!X1));
    }
    #[test]
    fn test_double_braced_double_negative_single_var() {
        let s = "((~~x1))";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(!!X1));
    }

    #[test]
    fn test_conjunction_of_two_vars() {
        let s = "x1 & x2";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(X1 & X2));
    }
    #[test]
    fn test_disjunction_of_two_vars() {
        let s = "x1 | x2";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(X1 | X2));
    }
    #[test]
    fn test_double_negated_conjunction_of_two_negated_vars() {
        let s = "~~(~x1 & ~x2)";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(!!(!X1 & !X2)));
    }

    #[test]
    fn test_conjunction_of_three_vars() {
        let s = "x1 & x2 & x3";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(X1 & X2 & X3));
    }
    #[test]
    fn test_disjunction_of_three_vars() {
        let s = "x1 | x2 | x3";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(X1 | X2 | X3));
    }

    #[test]
    fn test_mixed_expression() {
        let s = "x1 | x3 & (x2 | (~((x3))) ) & ~( x1 & ~~(x3 | (x1)) )";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(X1 | X3 & (X2 | !X3) & !(X1 & !!(X3 | X1))));
    }

    #[test]
    fn test_true() {
        let s = "true";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(TRUE));
    }
    #[test]
    fn test_false() {
        let s = "false";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(FALSE));
    }
    #[test]
    fn test_t() {
        let s = "T";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(TRUE));
    }
    #[test]
    fn test_f() {
        let s = "F";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(FALSE));
    }
    #[test]
    fn test_top() {
        let s = "⊤";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(TRUE));
    }
    #[test]
    fn test_bottom() {
        let s = "⊥";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(FALSE));
    }

    #[test]
    fn test_mixed_expression_with_constants() {
        let s = "⊤ | ~(F ) & (x2 | (~true & x1) | ⊥)";
        let expr = parse_expr(s);
        assert_eq!(expr, Ok(TRUE | !FALSE & (X2 | !TRUE & X1 | FALSE)));
    }

    #[test]
    fn test_mixed_expr_to_string() {
        let s = "(x1 | x2 & ~(x1 | x2 | ~~x3) | (x2) & ~x3)";
        let expr = parse_expr(s);
        assert!(expr.is_ok());
        assert_eq!(expr.unwrap().to_string(), "((1 | (2 & ~((1 | 2) | ~~3))) | (2 & ~3))");
    }

    #[test]
    fn test_implication() {
        let s = "x1 -> ~x2 => x3";
        let expr = parse_expr(s);
        assert!(expr.is_ok());
        assert_eq!(expr.unwrap().to_string(), "(1 -> (~2 -> 3))");
    }
}
