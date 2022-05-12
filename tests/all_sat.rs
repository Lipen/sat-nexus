use sat_nexus::core::lit::Lit;
use sat_nexus::core::op::allsat::AllSat;
use sat_nexus::core::solver::{Solver, SolverExt};
use sat_nexus::wrappers::ipasir::IpasirSolver;

#[test]
fn all_solutions_5vars() {
    let mut solver = IpasirSolver::new_cadical();

    let n = 5;
    let _lits = solver.new_var_vec(n);
    assert_eq!(solver.num_vars(), n);

    // Note: add the redundant clause `(x or -x)`, where x is the last used variable,
    //  in order to force the "allocation" of all variables inside the solver.
    solver.add_clause([Lit::from(n), -Lit::from(n)]);

    let num_solutions = solver.all_sat(|_| ()).count();
    assert_eq!(num_solutions, 32);
}

#[test]
fn all_solutions_essential_3of5vars() {
    let mut solver = IpasirSolver::new_cadical();

    let n = 5;
    let lits = solver.new_var_vec(n);
    assert_eq!(solver.num_vars(), n);

    // Note: add the redundant clause `(x or -x)`, where x is the last used variable,
    //  in order to force the "allocation" of all variables inside the solver.
    solver.add_clause([Lit::from(n), -Lit::from(n)]);

    let k = 3;
    let essential = lits[0..k].to_vec();
    let num_solutions = solver.all_sat_essential(essential, |_| ()).count();
    assert_eq!(num_solutions, 8);
}
