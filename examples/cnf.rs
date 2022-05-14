use color_eyre::eyre::Result;

use sat_nexus::core::cnf::Cnf;

fn main() -> Result<()> {
    color_eyre::install()?;

    let xs = [&[1, 2, 3][..], &[-3, 2][..]];
    println!("xs = {:?}", xs);
    let mut cnf = Cnf::from(xs);
    cnf.add_clause([-3, -1]);
    println!("cnf = {:?}", cnf);
    println!("cnf = {}", cnf);

    Ok(())
}
