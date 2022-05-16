use sat_nexus::core::context::Context;
use sat_nexus::core::solver::*;
use std::fmt::Display;

pub fn run_test_1<S>(mut solver: S) -> color_eyre::Result<()>
where
    S: Solver + Display,
{
    let mut context = Context::new();
    println!("Solver signature: {}", solver.signature());
    println!("solver = {}", solver);

    let value: i32 = 42;
    println!("value = {:?}", value);
    context.insert(value);
    let extracted = *context.extract::<i32>();
    println!("extracted = {:?}", extracted);

    solver.add_clause([1, 2]);
    solver.add_clause(vec![3, 4]);
    solver.add_clause([-1, -2]);
    solver.add_clause(vec![-3, -4]);
    solver.add_unit(5);
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
