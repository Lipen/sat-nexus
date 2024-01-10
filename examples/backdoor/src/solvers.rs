use crate::utils::clause_to_external;
use cadical::statik::Cadical;
use cadical::FixedResponse;
use simple_sat::lbool::LBool;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::var::Var;

#[derive(Debug)]
pub enum SatSolver {
    SimpleSat(Solver),
    Cadical(Cadical),
}

impl SatSolver {
    pub fn new_simple(solver: Solver) -> Self {
        SatSolver::SimpleSat(solver)
    }
    pub fn new_cadical(solver: Cadical) -> Self {
        SatSolver::Cadical(solver)
    }
}

impl SatSolver {
    pub fn num_vars(&self) -> u64 {
        match self {
            SatSolver::SimpleSat(solver) => solver.num_vars() as u64,
            SatSolver::Cadical(solver) => solver.vars() as u64,
        }
    }

    pub fn is_already_assigned(&self, var: Var) -> bool {
        match self {
            SatSolver::SimpleSat(solver) => solver.value_var(var) != LBool::Undef,
            SatSolver::Cadical(solver) => {
                let lit = var.to_external() as i32;
                solver.fixed(lit).unwrap() != FixedResponse::Unclear
            }
        }
    }

    pub fn is_active(&self, var: Var) -> bool {
        match self {
            SatSolver::SimpleSat(solver) => solver.value_var(var) != LBool::Undef,
            SatSolver::Cadical(solver) => {
                let lit = var.to_external() as i32;
                solver.is_active(lit)
            }
        }
    }

    pub fn add_clause(&mut self, lits: &[Lit]) {
        match self {
            SatSolver::SimpleSat(solver) => {
                solver.add_clause(lits);
            }
            SatSolver::Cadical(solver) => {
                solver.add_clause(clause_to_external(lits));
            }
        }
    }

    pub fn propcheck_all_tree(&mut self, vars: &[Var], limit: u64) -> u64 {
        match self {
            SatSolver::SimpleSat(solver) => {
                let mut out_learnts = Vec::new();
                solver.propcheck_all_tree(vars, limit, false, &mut out_learnts)
            }
            SatSolver::Cadical(solver) => {
                let vars_external: Vec<i32> = vars.iter().map(|var| var.to_external() as i32).collect();
                solver.propcheck_all_tree(&vars_external, limit)
            }
        }
    }
}
