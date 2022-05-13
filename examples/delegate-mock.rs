use color_eyre::eyre::Result;

use sat_nexus::core::context::Context;
use sat_nexus::core::solver::delegate::DelegateSolver;
use sat_nexus::core::solver::mock::MockSolver;
use sat_nexus::core::solver::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut solver = DelegateSolver::new(MockSolver::new());
    let mut context = Context::new();
    println!("Solver signature: {}", solver.signature());
    println!("solver = {}", solver);

    let value: i32 = 4;
    println!("value = {:?}", value);
    context.insert(value);
    let extracted = *context.extract::<i32>();
    println!("extracted = {:?}", extracted);

    solver.add_clause([1, 2]);
    solver.add_clause(&[3, 4]);
    solver.add_clause(vec![-1, -2]);
    solver.add_clause(&vec![-3, -4]);
    solver.add_unit(5);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    solver.assume(1);
    solver.assume(2);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    for i in 1..=5 {
        println!("solver.val({}) = {:?}", i, solver.value(i));
    }

    Ok(())
}
