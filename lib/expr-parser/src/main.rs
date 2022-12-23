use log::LevelFilter;
use simplelog::*;

// use expr_parser::nested::parse_expr;
use expr_parser::flat::parser::parse_expr;
use expr_parser::pomy::parse_expr as parse_expr_pom;

fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    for s in [
        "(x1 | ~(x2 | ~~~x4) & x3) | T & false",
        "x1 -> ~x2 | x3 -> x3 & x4",
        "x1 | x2 & x3 | x4",
    ] {
        println!("Input: {:?}", s);
        let expr = parse_expr(s);
        println!("Parsed: {:?}", expr);
        if let Ok(expr) = expr {
            println!("Parsed: {:#}", expr);
            println!("Parsed: {}", expr);
        }
        println!("==========");
    }

    let s = " x42 -> x1 or ~x2 and x0 <-> x99 ";
    println!("input: {:?}", s);
    let e2 = parse_expr_pom(s);
    println!("Parsed via pom: {:?}", e2);
}
