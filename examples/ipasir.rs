use std::convert::TryInto;

use color_eyre::eyre::Result;

use ipasir::Ipasir;
use sat_nexus::ipasir::IpasirSolver;

fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    let solver = IpasirSolver::new_cadical();
    println!("Solver signature: {}", solver.signature());

    solver.try_add_clause(&[1, 2])?;
    solver.try_add_clause(&[3, 4])?;
    solver.try_add_clause(&[-1, -2])?;
    solver.try_add_clause(&[-3, -4])?;
    let response = solver.solve()?;
    println!("Solver returned: {:?}", response);

    solver.assume(1.try_into()?);
    solver.assume(2.try_into()?);
    let response = solver.solve()?;
    println!("Solver returned: {:?}", response);

    // solver.assume(0.try_into()?);
    let response = solver.solve()?;
    println!("Solver returned: {:?}", response);

    Ok(())
}
