use log::debug;
use pest::error::Error;
use pest::Parser;

use super::expr::Expr;

#[derive(Parser)]
#[grammar = "grammar/nested.pest"] // relative to project `src`
struct ExprParser;

pub fn parse_expr(input: &str) -> Result<Expr, Error<Rule>> {
    let expression = ExprParser::parse(Rule::main, input)?.next().unwrap();

    use pest::iterators::Pair;

    fn parse_expression(expr: Pair<Rule>) -> Expr {
        debug!("expr = {} = {}", expr.as_str(), expr);
        let d = expr.into_inner().next().unwrap();
        parse_disjunction(d)
    }

    fn parse_disjunction(disj: Pair<Rule>) -> Expr {
        debug!("disj = {} = {}", disj.as_str(), disj);
        Expr::or(disj.into_inner().map(parse_conjunction))
    }

    fn parse_conjunction(conj: Pair<Rule>) -> Expr {
        debug!("conj = {} = {}", conj.as_str(), conj);
        Expr::and(conj.into_inner().map(parse_term))
    }

    fn parse_term(term: Pair<Rule>) -> Expr {
        debug!("term = {:?} = {}", term.as_str(), term);
        match term.as_rule() {
            Rule::negated_term => {
                let t = term.into_inner().next().unwrap();
                Expr::not(parse_term(t))
            }
            Rule::braced_expression => {
                let e = term.into_inner().next().unwrap();
                parse_expression(e)
            }
            Rule::variable => {
                let v: u32 = term.as_str()[1..].parse().unwrap();
                Expr::Var(v)
            }
            Rule::bool => {
                let b = term.into_inner().next().unwrap();
                match b.as_rule() {
                    Rule::true_lit => Expr::Const(true),
                    Rule::false_lit => Expr::Const(false),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(parse_expression(expression))
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
        assert_eq!(expr.map(|x| x.to_string()), Ok(X1.to_string()));
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
}
