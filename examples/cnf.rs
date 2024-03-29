use sat_nexus_core::cnf::Cnf;
use sat_nexus_core::op::ops::AddClause;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let xs = [&[1, 2, 3][..], &[-3, 2][..]];
    println!("xs = {:?}", xs);
    let mut cnf = Cnf::from_iter(xs);
    cnf.add_clause([-3, -1]);
    println!("cnf = {:?}", cnf);
    println!("cnf = {}", cnf);

    Ok(())
}
