use itertools::Itertools;

use sat_nexus::core::card::Cardinality;
use sat_nexus::core::op::allsat::AllSat;
use sat_nexus::core::op::ops::Ops;
use sat_nexus::core::solver::{Solver, SolverExt};
use sat_nexus::wrappers::ipasir::WrappedIpasirSolver;

#[test]
fn test_totalizer() {
    let mut solver = WrappedIpasirSolver::new_cadical();
    let n = 8;
    let ub = 6;
    let lb = 2;
    let lits = solver.new_var_vec(n);

    for i in 0..(n - 1) {
        solver.imply(lits[i], lits[i + 1]);
    }

    let mut totalizer = solver.declare_totalizer(&lits);
    totalizer.declare_upper_bound_less_than_or_equal(&mut solver, ub);
    totalizer.declare_lower_bound_greater_than_or_equal(&mut solver, lb);

    let mut num_solutions = 0;
    solver
        .all_sat_essential(lits.clone(), |solver| {
            lits.iter().map(|&x| solver.value(x)).collect_vec()
        })
        .for_each(|solution| {
            num_solutions += 1;
            println!(
                "Solution #{}: {:?} == {}",
                num_solutions,
                solution,
                solution.iter().filter(|x| x.bool()).count()
            );
        });
    assert_eq!(num_solutions, ub - lb + 1);
}
