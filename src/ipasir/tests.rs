use std::convert::TryInto;

use solver::IpasirSolver;

use super::*;

#[test]
fn test_ipasir_solver() -> color_eyre::Result<()> {
    let solver = IpasirSolver::new_cadical();
    assert!(solver.signature().starts_with("cadical"));

    // Adding [(1 or 2) and (3 or 4) and not(1 and 2) and not(3 and 4)]
    solver.try_add_clause([1, 2])?;
    solver.try_add_clause([3, 4])?;
    solver.try_add_clause([-1, -2])?;
    solver.try_add_clause([-3, -4])?;

    // Problem is satisfiable
    let response = solver.solve()?;
    assert_eq!(response, SolveResponse::Sat);

    // Assuming both 1 and 2 to be true
    solver.assume(1.try_into()?);
    solver.assume(2.try_into()?);
    // Problem is unsatisfiable under assumptions
    let response = solver.solve()?;
    assert_eq!(response, SolveResponse::Unsat);

    // `solve` resets assumptions, so calling it again should produce SAT
    let response = solver.solve()?;
    assert_eq!(response, SolveResponse::Sat);

    Ok(())
}
