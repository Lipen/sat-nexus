use log::debug;

use crate::cnf::Cnf;
use crate::solver::Solver;

pub fn bootstrap_solver_from_cnf(solver: &mut impl Solver, cnf: &Cnf) {
    if cnf.max_var > solver.num_vars() {
        debug!("Adding {} variables...", cnf.max_var - solver.num_vars());
        for _ in solver.num_vars()..cnf.max_var {
            solver.new_var();
        }
    }

    debug!("Adding {} clauses...", cnf.clauses.len());
    for clause in cnf.clauses.iter() {
        solver.add_clause(&clause.lits)
    }
}

pub trait TypeName {
    fn type_name(&self) -> String;
}

impl<T: ?Sized> TypeName for T {
    fn type_name(&self) -> String {
        tynm::type_name::<T>()
    }
}
