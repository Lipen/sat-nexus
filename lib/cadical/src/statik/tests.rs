use super::*;
use crate::common::*;

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
    solver.assume(1)?;
    solver.assume(2)?;
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
    println!("values: {:?}", vec![val1, val2, val3, val4]);
    assert!(bool::from(val1) ^ bool::from(val2));
    assert!(bool::from(val3) ^ bool::from(val4));

    println!("conflicts:    {}", solver.conflicts());
    println!("decisions:    {}", solver.decisions());
    println!("restarts:     {}", solver.restarts());
    println!("propagations: {}", solver.propagations());

    Ok(())
}

#[test]
fn test_simple_unsat() -> color_eyre::Result<()> {
    let solver = Cadical::new();

    solver.add_clause([1]);
    solver.add_clause([-2]);
    solver.assume(-1)?;
    let res = solver.solve()?;
    assert_eq!(res, SolveResponse::Unsat);

    let f1 = solver.failed(1)?;
    let fn1 = solver.failed(-1)?;
    println!("failed 1: {}, -1: {}", f1, fn1);
    assert!(fn1);
    assert!(!f1);
    let f2 = solver.failed(2)?;
    let fn2 = solver.failed(-2)?;
    println!("failed 2: {}, -2: {}", f2, fn2);

    println!("active: {}", solver.active());
    println!("irredundant: {}", solver.irredundant());
    println!("fixed 1: {:?}, fixed -1: {:?}", solver.fixed(1)?, solver.fixed(-1)?);
    println!("fixed 2: {:?}, fixed -2: {:?}", solver.fixed(2)?, solver.fixed(-2)?);
    println!("fixed 3: {:?}, fixed -3: {:?}", solver.fixed(3)?, solver.fixed(-3)?);

    println!("frozen 1: {}, frozen 2: {}", solver.frozen(1)?, solver.frozen(2)?);
    solver.freeze(2)?;
    println!("frozen 1: {}, frozen 2: {}", solver.frozen(1)?, solver.frozen(2)?);
    assert!(solver.frozen(2)?);
    solver.melt(2)?;
    println!("frozen 1: {:?}, frozen 2: {:?}", solver.frozen(1)?, solver.frozen(2)?);
    assert!(!solver.frozen(2)?);

    let res = solver.simplify()?;
    println!("simplify() = {:?}", res);

    Ok(())
}

#[test]
fn test_learner() {
    let solver = Cadical::new();
    println!("solver = {:?}", solver);

    solver.set_option("otfs", 0);

    let mut learnts: Vec<Vec<i32>> = Vec::new();

    println!("Setting learner...");
    solver.set_learn(0, |clause| {
        println!("learned clause: {:?}", clause);
        learnts.push(clause);
    });

    println!("Adding clauses...");
    for r in [-1, 1].iter() {
        for s in [-1, 1].iter() {
            for t in [-1, 1].iter() {
                solver.add_clause([r * 1, s * 2, t * 3]);
            }
        }
    }

    println!("vars: {}", solver.vars());
    println!("clauses: {}", solver.irredundant());
    println!("learnts = {:?}", learnts);

    println!("Solving...");
    let res = solver.solve();
    println!("res = {:?}", res);

    println!("learnts = {:?}", learnts);
}

#[test]
fn test_traverse_clauses() {
    let solver = Cadical::new();
    println!("solver = {:?}", solver);

    solver.add_clause([1, -2]);
    solver.add_clause([-3, 4]);
    solver.add_clause([5, -6, -7]);

    let mut clauses = Vec::new();
    let res = solver.traverse_clauses(false, |clause| {
        let clause = clause.to_vec();
        println!("clause: {:?}", clause);
        clauses.push(clause);
        true
    });
    assert!(res);

    println!("Total {} clauses: {:?}", clauses.len(), clauses);
    assert_eq!(clauses.len(), 3);
}
