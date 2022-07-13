use color_eyre::Result;
use sat_nexus_core::formula::expr::Expr;
use sat_nexus_core::formula::var::Var;

fn main() -> Result<()> {
    color_eyre::install()?;

    let s = "x1 | x2 & !x3";
    println!("Input: {}", s);
    let expr = Expr::parse(s);
    println!("Parsed: {:?}", expr);
    if let Ok(expr) = expr {
        println!("Parsed: {}", expr);
        println!("Parsed: {:#}", expr);
    }

    let x1 = Var(1);
    let x2 = Var(2);
    let x3 = Var(3);
    let e: Expr = x1 | x2 & x3;
    println!("e = {} = {0:?}", e);

    Ok(())
}
