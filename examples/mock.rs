use color_eyre::eyre::Result;

use sat_nexus::core::solver::mock::MockSolver;
use sat_nexus::core::solver::Solver;

fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    let mut solver = MockSolver::new();
    println!("Solver signature: {}", solver.signature());
    println!("solver = {}", solver);

    let shared_context = solver.context();
    let mut context = shared_context.borrow_mut();
    let value: i32 = 4;
    println!("value = {:?}", value);
    context.insert(value);
    let extracted = *context.extract::<i32>();
    println!("extracted = {:?}", extracted);

    solver.add_clause(&[1, 2]);
    solver.add_clause(&[3, 4]);
    solver.add_clause(&[-1, -2]);
    solver.add_clause(&[-3, -4]);
    solver.add_clause(&[5, -5]);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    solver.assume(1);
    solver.assume(2);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    for i in 1..=5 {
        println!("solver.val({}) = {:?}", i, solver.val(i));
    }

    Ok(())
}
