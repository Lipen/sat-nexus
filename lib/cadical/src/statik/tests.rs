use super::*;

use std::cell::RefCell;
use std::rc::Rc;

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
    solver.unsafe_set_learn(0, |clause| {
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
fn test_learner2() {
    struct Wrapper {
        solver: Cadical,
        learnts: Rc<RefCell<Vec<Vec<i32>>>>,
    }

    impl Wrapper {
        fn new() -> Self {
            let solver = Cadical::new();
            println!("solver = {:?}", solver);

            solver.set_option("otfs", 0);

            let learnts = Vec::new();
            let learnts = Rc::new(RefCell::new(learnts));
            {
                println!("Setting learner...");
                let learnts = Rc::clone(&learnts);
                solver.set_learn(0, move |clause| {
                    println!("learned clause: {:?}", clause);
                    learnts.borrow_mut().push(clause);
                });
            }

            Self { solver, learnts }
        }
    }

    let wrapper = Wrapper::new();

    println!("Adding clauses...");
    for r in [-1, 1].iter() {
        for s in [-1, 1].iter() {
            for t in [-1, 1].iter() {
                wrapper.solver.add_clause([r * 1, s * 2, t * 3]);
            }
        }
    }

    println!("vars: {}", wrapper.solver.vars());
    println!("clauses: {}", wrapper.solver.irredundant());
    println!("learnts = {:?}", wrapper.learnts);

    println!("Solving...");
    let res = wrapper.solver.solve();
    println!("res = {:?}", res);

    println!("learnts = {:?}", wrapper.learnts);
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

#[test]
fn test_top_score_variables() {
    let solver = Cadical::new();
    println!("solver = {:?}", solver);

    println!("Adding clauses...");

    let n: usize = 10; // number of holes for (n+1) pigeons
    println!("Encoding PHP({}, {})...", n + 1, n);

    fn pigeon_in_hole(p: usize, h: usize, n: usize) -> i32 {
        let p = p as i32;
        let h = h as i32;
        let n = n as i32;
        n * (p - 1) + h
    }

    for p in 1..=(n + 1) {
        let mut clause = Vec::new();
        for h in 1..=n {
            clause.push(pigeon_in_hole(p, h, n));
        }
        solver.add_clause(clause);
    }

    for h in 1..=n {
        for p1 in 1..=(n + 1) {
            let v1 = pigeon_in_hole(p1, h, n);
            for p2 in (p1 + 1)..=(n + 1) {
                let v2 = pigeon_in_hole(p2, h, n);
                solver.add_clause(vec![-v1, -v2]);
            }
        }
    }

    // for p in 1..=(n + 1) {
    //     for h in 1..=n {
    //         println!("pigeon_in_hole({}, {}) = {}", p, h, pigeon_in_hole(p, h, n));
    //     }
    // }

    solver.limit("conflicts", 1000);
    let res = solver.solve().unwrap();
    println!("res = {:?}", res);

    let limit = 100;
    let top_score_vars = solver.get_top_score_variables(limit);
    println!("Top {} vars with highest score: {:?}", limit, top_score_vars);
}
