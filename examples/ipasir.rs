use std::convert::TryInto;

use color_eyre::eyre::Result;

use sat_nexus::ipasir::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    let solver = Ipasir::new_cadical();
    println!("Solver signature: {}", solver.signature());

    solver.try_add_clause(&[1, 2])?;
    solver.try_add_clause(&[3, 4])?;
    solver.try_add_clause(&[-1, -2])?;
    solver.try_add_clause(&[-3, -4])?;
    let response = solver.solve()?;
    println!("Solver returned: {:?}", response);
    assert!(matches!(response, SolveResponse::Sat));

    solver.assume(1.try_into()?);
    solver.assume(2.try_into()?);
    let response = solver.solve()?;
    println!("Solver returned: {:?}", response);
    assert!(matches!(response, SolveResponse::Unsat));

    let response = solver.solve()?;
    println!("Solver returned: {:?}", response);
    assert!(matches!(response, SolveResponse::Sat));

    Ok(())
}
