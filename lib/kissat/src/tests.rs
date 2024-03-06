use super::*;

#[test]
fn test_kissat_solver() {
    let solver = Kissat::new();
    println!("solver = {}", solver);
    assert!(solver.signature().contains("kissat"));

    // Encoding (x1 xor x2) and (x3 xor x4)
    // Adding [(1 or 2) and (3 or 4) and not(1 and 2) and not(3 and 4)]
    solver.add_clause([1, 2]);
    solver.add_clause(vec![3, 4]);
    solver.add_clause([-1, -2]);
    solver.add_clause(vec![-3, -4]);

    // Problem is satisfiable
    let response = solver.solve();
    assert_eq!(response, SolveResponse::Sat);

    let val1 = solver.value(1);
    let val2 = solver.value(2);
    let val3 = solver.value(3);
    let val4 = solver.value(4);
    println!("values: {:?}", vec![val1, val2, val3, val4]);
    assert!(bool::from(val1) ^ bool::from(val2));
    assert!(bool::from(val3) ^ bool::from(val4));
}
