use color_eyre::eyre::Result;

use sat_nexus::core::context::Context;
use sat_nexus::core::solver::*;
use sat_nexus::wrappers::minisat::MiniSatSolver;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut solver = MiniSatSolver::new();
    let mut context = Context::new();
    println!("Solver signature: {}", solver.signature());
    println!("solver = {}", solver);

    let value: i32 = 4;
    println!("value = {:?}", value);
    context.insert(value);
    let extracted = *context.extract::<i32>();
    println!("extracted = {:?}", extracted);

    solver.add_clause([1, 2]);
    solver.add_clause(vec![3, 4]);
    solver.add_clause([-1, -2]);
    solver.add_clause(vec![-3, -4]);
    solver.add_clause(&[5, -5]);
    assert_eq!(5, solver.num_vars());
    println!("num_vars = {}", solver.num_vars());
    let response = solver.solve();
    println!("Solver returned: {:?}", response);
    assert!(matches!(response, SolveResponse::Sat));

    solver.assume(1);
    solver.assume(2);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);
    assert!(matches!(response, SolveResponse::Unsat));

    let response = solver.solve();
    println!("Solver returned: {:?}", response);
    assert!(matches!(response, SolveResponse::Sat));

    for i in 1..=5 {
        println!("solver.val({}) = {:?}", i, solver.value(i));
    }

    Ok(())
}
