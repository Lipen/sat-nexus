use minisat::dynamic::*;

//noinspection DuplicatedCode
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let solver = MiniSat::new();
    println!("Solver signature: {}", solver.signature());
    println!("solver = {}", solver);

    let lit1 = solver.new_lit();
    let lit2 = solver.new_lit();
    let lit3 = solver.new_lit();
    let lit4 = solver.new_lit();
    let lit5 = solver.new_lit();

    solver.add_clause([lit1, lit2]);
    solver.add_clause(vec![lit3, lit4]);
    solver.try_add_clause([-lit1, -lit2])?;
    solver.try_add_clause(vec![-lit3, -lit4])?;
    solver.try_add_clause([lit5, -lit5])?;

    // Solving without assumptions => SAT
    let response = solver.solve();
    println!("Solver returned: {}", response);
    assert!(response);

    // Solving with assumptions => UNSAT
    let response = solver.solve_under_assumptions([lit1, lit2]);
    println!("Solver returned: {}", response);
    assert!(!response);

    // Solving again without assumptions => SAT
    let response = solver.solve();
    println!("Solver returned: {}", response);
    assert!(response);

    for lit in [lit1, lit2, lit3, lit4, lit5] {
        println!("solver.val({}) = {:?}", lit, solver.model_value_lit(lit));
    }

    Ok(())
}
