use std::convert::TryInto;

use solver::CadicalSolver;

use crate::ipasir::{LitValue, SolveResponse};

use super::*;

fn lit_value_to_bool(value: LitValue) -> bool {
    match value {
        LitValue::True => true,
        LitValue::False => false,
        LitValue::DontCare => panic!("DontCare!"),
    }
}

#[test]
fn test_cadical_solver() -> color_eyre::Result<()> {
    let solver = CadicalSolver::new();
    assert!(solver.signature().starts_with("cadical"));

    // Adding [(1 or 2) and (3 or 4) and not(1 and 2) and not(3 and 4)]
    solver.try_add_clause(&[1, 2])?;
    solver.try_add_clause(&[3, 4])?;
    solver.try_add_clause(&[-1, -2])?;
    solver.try_add_clause(&[-3, -4])?;

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

    let val1 = lit_value_to_bool(solver.val(1.try_into()?)?);
    let val2 = lit_value_to_bool(solver.val(2.try_into()?)?);
    let val3 = lit_value_to_bool(solver.val(3.try_into()?)?);
    let val4 = lit_value_to_bool(solver.val(4.try_into()?)?);
    eprintln!("values: {:?}", vec![val1, val2, val3, val4]);
    assert!(val1 ^ val2);
    assert!(val3 ^ val4);

    Ok(())
}
