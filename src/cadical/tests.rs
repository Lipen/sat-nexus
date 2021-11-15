use std::convert::TryInto;

use less::CadicalSolver2;
use solver::CadicalSolver;

use crate::ipasir::SolveResponse;

use super::*;

#[test]
fn test_cadical_solver() -> color_eyre::Result<()> {
    let solver = CadicalSolver::new();
    assert!(solver.signature().starts_with("cadical"));

    // Adding [(1 or 2) and (3 or 4) and not(1 and 2) and not(3 and 4)]
    solver.try_add_clause([1, 2])?;
    solver.try_add_clause(&[3, 4])?;
    solver.try_add_clause(vec![-1, -2])?;
    solver.try_add_clause(&vec![-3, -4])?;

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

    let val1 = solver.val(1.try_into()?)?.bool();
    let val2 = solver.val(2.try_into()?)?.bool();
    let val3 = solver.val(3.try_into()?)?.bool();
    let val4 = solver.val(4.try_into()?)?.bool();
    eprintln!("values: {:?}", vec![val1, val2, val3, val4]);
    assert!(val1 ^ val2);
    assert!(val3 ^ val4);

    Ok(())
}

#[test]
fn test_cadical_solver2() -> color_eyre::Result<()> {
    let solver = CadicalSolver2::new();
    assert!(solver.signature().starts_with("cadical"));

    // Adding [(1 or 2) and (3 or 4) and not(1 and 2) and not(3 and 4)]
    solver.add_clause([1, 2]);
    solver.add_clause(*&[3, 4]);
    solver.add_clause(vec![-1, -2]);
    solver.add_clause([-3, -4]);

    // Problem is satisfiable
    let response = solver.solve();
    assert_eq!(response, 10); // 10 is SAT, 20 is UNSAT

    // Assuming both 1 and 2 to be true
    solver.assume(1);
    solver.assume(2);
    // Problem is unsatisfiable under assumptions
    let response = solver.solve();
    assert_eq!(response, 20); // 10 is SAT, 20 is UNSAT

    // `solve` resets assumptions, so calling it again should produce SAT
    let response = solver.solve();
    assert_eq!(response, 10); // 10 is SAT, 20 is UNSAT

    let val1 = solver.val(1);
    let val2 = solver.val(2);
    let val3 = solver.val(3);
    let val4 = solver.val(4);
    eprintln!("values: {:?}", vec![val1, val2, val3, val4]);
    assert!((val1 > 0) ^ (val2 > 0));
    assert!((val3 > 0) ^ (val4 > 0));

    Ok(())
}
