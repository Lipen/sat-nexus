use cadical::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let solver = Cadical::new();
    println!("solver = {}", solver);

    // ------------------------------------------------------------------
    // Encode Problem and check without assumptions.

    const TIE: i32 = 1;
    const SHIRT: i32 = 2;
    solver.add_clause([-TIE, SHIRT]);
    solver.add_clause([TIE, SHIRT]);
    solver.add_clause([-TIE, -SHIRT]);

    // Problem is satisfiable (without assumptions).
    let res = solver.solve()?;
    assert_eq!(res, SolveResponse::Sat);

    // Check TIE is false, SHIRT is true.
    assert_eq!(solver.val(TIE)?, LitValue::False);
    assert_eq!(solver.val(SHIRT)?, LitValue::True);

    // ------------------------------------------------------------------
    // Incrementally solve again under one assumption.

    // Force TIE to true.
    solver.assume(TIE);

    // Problem is now unsatisfiable (under assumptions).
    let res = solver.solve()?;
    assert_eq!(res, SolveResponse::Unsat);

    // Check that TIE is responsible for this:
    //   Yes, `TIE` is in unsat core.
    assert!(solver.failed(TIE)?);
    // Check that SHIRT is responsible for this:
    //   No, SHIRT is NOT in unsat core.
    assert!(!solver.failed(SHIRT)?);

    // ------------------------------------------------------------------
    // Incrementally solve with constraint.

    // Add constraint (TIE,-SHIRT)
    solver.constrain(TIE);
    solver.constrain(-SHIRT);
    solver.constrain(0);

    // Problem is unsatisfiable again.
    let res = solver.solve()?;
    assert_eq!(res, SolveResponse::Unsat);

    // Check constraint is responsible for this.
    assert!(solver.constraint_failed()?);

    // ------------------------------------------------------------------
    // Incrementally solve once more under another assumption.

    // Now assume SHIRT is false.
    solver.assume(-SHIRT);

    // Problem is unsatisfiable once again.
    let res = solver.solve()?;
    assert_eq!(res, SolveResponse::Unsat);

    // Check that TIE is responsible for this:
    //   No, `TIE` is NOT in unsat core.
    assert!(!solver.failed(TIE)?);
    // Check that ~SHIRT is responsible for this:
    //   Yes, ~SHIRT is in unsat core.
    assert!(solver.failed(-SHIRT)?);

    // ------------------------------------------------------------------

    println!("OK!");
    Ok(())
}
