use log::LevelFilter;
use simplelog::*;

// use expr_parser::nested::parse_expr;
use expr_parser::flat::parser::parse_expr;

fn main() {
    TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    // let s = "(x1 | ~(x2 | ~~~x4) & x3) | T & false";
    // let s = "x1 -> ~x2 | x3 -> x3 & x4";
    let s = "x1 | x2 & x3 | x4";
    println!("Input: {:?}", s);
    let expr = parse_expr(s);
    println!("Parsed: {:?}", expr);
    if let Ok(expr) = expr {
        println!("Parsed: {:#}", expr);
        println!("Parsed: {}", expr);
    }
}
