use color_eyre::eyre::Result;

use sat_nexus::cadical::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    let solver = Cadical::new();
    println!("Solver signature: {}", solver.signature());
    println!("solver = {}", solver);

    solver.add_clause([1, 2]);
    solver.add_clause(vec![3, 4]);
    solver.add_clause([-1, -2]);
    solver.add_clause(vec![-3, -4]);
    solver.add_clause([5, -5]);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);
    assert!(matches!(response, Ok(SolveResponse::Sat)));

    solver.assume(1);
    solver.assume(2);
    let response = solver.solve()?;
    println!("Solver returned: {:?}", response);

    let response = solver.solve()?;
    println!("Solver returned: {:?}", response);

    for i in 1..=5 {
        println!("solver.val({}) = {:?}", i, solver.val(i)?);
    }

    Ok(())
}
