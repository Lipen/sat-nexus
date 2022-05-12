use super::*;

#[test]
fn test_cadical_solver() -> color_eyre::Result<()> {
    let solver = Cadical::new();
    assert!(solver.signature().contains("cadical"));

    // Adding [(1 or 2) and (3 or 4) and not(1 and 2) and not(3 and 4)]
    solver.add_clause([1, 2]);
    solver.add_clause(vec![3, 4]);
    solver.try_add_clause([-1, -2])?;
    solver.try_add_clause(vec![-3, -4])?;

    // Problem is satisfiable
    let response = solver.solve()?;
    assert_eq!(response, SolveResponse::Sat);

    // Assuming both 1 and 2 to be true
    solver.assume(1);
    solver.assume(2);
    // Problem is unsatisfiable under assumptions
    let response = solver.solve()?;
    assert_eq!(response, SolveResponse::Unsat);

    // `solve` resets assumptions, so calling it again should produce SAT
    let response = solver.solve()?;
    assert_eq!(response, SolveResponse::Sat);

    let val1 = solver.val(1)?;
    let val2 = solver.val(2)?;
    let val3 = solver.val(3)?;
    let val4 = solver.val(4)?;
    eprintln!("values: {:?}", vec![val1, val2, val3, val4]);
    assert!(bool::from(val1) ^ bool::from(val2));
    assert!(bool::from(val3) ^ bool::from(val4));

    Ok(())
}
