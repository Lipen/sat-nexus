#![allow(unused)]

use std::str::FromStr;

use pom::parser::{call, end, list, one_of, seq, sym, tag, Parser};

#[derive(Debug)]
pub enum Expr {
    Const(bool),
    Var(u32),
    Not { arg: Box<Expr> },
    And { args: Vec<Expr> },
    Or { args: Vec<Expr> },
    Imply { lhs: Box<Expr>, rhs: Box<Expr> },
    Iff { lhs: Box<Expr>, rhs: Box<Expr> },
}

fn space<'a>() -> Parser<'a, char, ()> {
    one_of(" \t\r\n").repeat(0..).discard()
}

fn constant<'a>() -> Parser<'a, char, bool> {
    tag("true").map(|_| true) | tag("false").map(|_| false)
}

fn var<'a>() -> Parser<'a, char, u32> {
    let integer = one_of("123456789") - one_of("0123456789").repeat(0..) | sym('0');
    let number = integer.collect().map(String::from_iter).convert(|s| u32::from_str(&s));
    sym('x') * number
}

fn negated_term<'a>() -> Parser<'a, char, Expr> {
    (sym('~') | sym('!')) * space() * call(term)
}

fn braced_expr<'a>() -> Parser<'a, char, Expr> {
    sym('(') * space() * call(expr) - space() - sym(')')
}

fn term<'a>() -> Parser<'a, char, Expr> {
    (negated_term().map(|t| Expr::Not { arg: Box::new(t) })
        | braced_expr()
        | var().map(|v| Expr::Var(v))
        | constant().map(|b| Expr::Const(b)))
        - space()
}

fn conjunction<'a>() -> Parser<'a, char, Expr> {
    // let sep = sym('&');
    let sep = tag("&") | tag("and");
    list(term(), sep - space()).map(|mut args| {
        assert!(!args.is_empty());
        if args.len() > 1 {
            Expr::And { args }
        } else {
            args.remove(0)
        }
    })
}

fn disjunction<'a>() -> Parser<'a, char, Expr> {
    // let sep = sym('|');
    let sep = tag("|") | tag("or");
    list(conjunction(), sep - space()).map(|mut args| {
        assert!(!args.is_empty());
        if args.len() > 1 {
            Expr::Or { args }
        } else {
            args.remove(0)
        }
    })
}

fn imply<'a>() -> Parser<'a, char, Expr> {
    let sep = tag("->") | tag("=>");
    list(disjunction(), sep - space()).map(|args| {
        args.into_iter()
            .rev()
            .reduce(|a, b| Expr::Imply {
                lhs: Box::new(b),
                rhs: Box::new(a),
            })
            .unwrap()
    })
}

fn iff<'a>() -> Parser<'a, char, Expr> {
    let sep = tag("<->") | tag("<=>");
    list(imply(), sep - space()).map(|args| {
        args.into_iter()
            .reduce(|a, b| Expr::Iff {
                lhs: Box::new(a),
                rhs: Box::new(b),
            })
            .unwrap()
    })
}

fn expr<'a>() -> Parser<'a, char, Expr> {
    space() * iff() - end()
}

pub fn parse_expr(input: &str) -> pom::Result<Expr> {
    let input: Vec<char> = input.chars().collect();
    let parser = expr();
    parser.parse(&input) //.expect("Could not parse")
}
