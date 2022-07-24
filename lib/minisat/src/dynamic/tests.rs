//noinspection Duplicates
use super::*;

#[test]
fn test_minisat_solver_dynamic() -> color_eyre::Result<()> {
    let solver = MiniSat::new();
    println!("solver = {}", solver);
    assert!(solver.signature().starts_with("minisat"));

    let lit1 = solver.new_lit();
    let lit2 = solver.new_lit();
    let lit3 = solver.new_lit();
    let lit4 = solver.new_lit();

    // Adding [(1 or 2) and (3 or 4) and not(1 and 2) and not(3 and 4)]
    solver.add_clause([lit1, lit2]);
    solver.add_clause(vec![lit3, lit4]);
    solver.try_add_clause([-lit1, -lit2])?;
    solver.try_add_clause(vec![-lit3, -lit4])?;

    // Problem is satisfiable
    let res = solver.solve();
    assert!(res);

    // Problem is unsatisfiable under assumptions (1 and 2)
    let res = solver.solve_under_assumptions([lit1, lit2]);
    assert!(!res);

    // `solve` resets assumptions, so calling it again should produce SAT
    let res = solver.solve();
    assert!(res);

    let val1 = solver.model_value_lit(lit1);
    let val2 = solver.model_value_lit(lit2);
    let val3 = solver.model_value_lit(lit3);
    let val4 = solver.model_value_lit(lit4);
    println!("values: {:?}", [val1, val2, val3, val4]);
    assert!(val1.bool() ^ val2.bool());
    assert!(val3.bool() ^ val4.bool());

    println!("Statistics:");
    println!("vars =         {}", solver.num_vars());
    println!("clauses =      {}", solver.num_clauses());
    println!("assigns =      {}", solver.num_assigns());
    println!("free_vars =    {}", solver.num_free_vars());
    println!("learnts =      {}", solver.num_learnts());
    println!("conflicts =    {}", solver.num_conflicts());
    println!("decisions =    {}", solver.num_decisions());
    println!("restarts =     {}", solver.num_restarts());
    println!("propagations = {}", solver.num_propagations());

    println!("{}", "=".repeat(42));
    Ok(())
}
