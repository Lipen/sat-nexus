use kissat::statik::*;

//noinspection DuplicatedCode
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let solver = Kissat::new();
    println!("Solver signature: {}", solver.signature());
    println!("solver = {}", solver);

    let mut num_vars = 0;
    let mut new_var = || {
        num_vars += 1;
        num_vars
    };

    let lit1 = new_var();
    let lit2 = new_var();
    let lit3 = new_var();
    let lit4 = new_var();
    let lit5 = new_var();

    solver.add_clause([lit1, lit2]);
    solver.add_clause(vec![lit3, lit4]);
    solver.add_clause([-lit1, -lit2]);
    solver.add_clause(vec![-lit3, -lit4]);
    solver.add_clause(vec![lit5, -lit5]);

    // Solving without assumptions => SAT
    let response = solver.solve();
    println!("Solver returned: {:?}", response);
    assert_eq!(response, SolveResponse::Sat);

    // Note: Kissat currently does not support assumptions.
    //
    // // Solving with assumptions => UNSAT
    // solver.assume(lit1);
    // solver.assume(lit2);
    // let response = solver.solve();
    // println!("Solver returned: {:?}", response);
    // assert_eq!(response, SolveResponse::Unsat);
    //
    // // Solving again without assumptions => SAT
    // let response = solver.solve();
    // println!("Solver returned: {:?}", response);
    // assert_eq!(response, SolveResponse::Sat);

    for lit in [lit1, lit2, lit3, lit4, lit5] {
        println!("solver.val({}) = {:?}", lit, solver.value(lit));
    }

    Ok(())
}
