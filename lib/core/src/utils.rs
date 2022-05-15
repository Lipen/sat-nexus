use log::info;

use crate::cnf::Cnf;
use crate::solver::Solver;

pub fn bootstrap_solver_from_cnf(solver: &mut impl Solver, cnf: &Cnf) {
    if cnf.max_var > solver.num_vars() {
        info!("Adding {} variables...", cnf.max_var - solver.num_vars());
        for _ in solver.num_vars()..cnf.max_var {
            solver.new_var();
        }
    }

    info!("Adding {} clauses...", cnf.clauses.len());
    for clause in cnf.clauses.iter() {
        solver.add_clause(&clause.lits)
    }
}

pub fn type_name_of<T: ?Sized>(_val: &T) -> String {
    tynm::type_name::<T>()
}

pub trait TypeName {
    fn type_name(&self) -> String;
}

impl<T: ?Sized> TypeName for T {
    fn type_name(&self) -> String {
        type_name_of(self)
    }
}
