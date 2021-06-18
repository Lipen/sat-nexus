use std::convert::TryInto;

use color_eyre::eyre::Result;

use sat_nexus::cadical::solver::CadicalSolver;
use sat_nexus::cadical::CadicalInterface;

fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    let solver = CadicalSolver::new();
    println!("Solver signature: {}", solver.signature());
    println!("solver = {}", solver);

    // let context = solver.context_mut();
    // let value: i32 = 4;
    // println!("value = {:?}", value);
    // context.insert(value);
    // let extracted = *context.extract::<i32>();
    // println!("extracted = {:?}", extracted);

    solver.try_add_clause(&[1, 2])?;
    solver.try_add_clause(&[3, 4])?;
    solver.try_add_clause(&[-1, -2])?;
    solver.try_add_clause(&[-3, -4])?;
    solver.try_add_clause(&[5, -5])?;
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    solver.assume(1.try_into()?);
    solver.assume(2.try_into()?);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    for i in 1..=5 {
        println!("solver.val({}) = {:?}", i, solver.val(i.try_into()?));
    }

    Ok(())
}
