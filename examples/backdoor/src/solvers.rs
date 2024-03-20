use cadical::statik::Cadical;
use cadical::FixedResponse;
use itertools::Itertools;
use simple_sat::lbool::LBool;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::utils::DisplaySlice;
use simple_sat::var::Var;

use crate::utils::clause_to_external;

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
            SatSolver::SimpleSat(solver) => solver.value_var(var) == LBool::Undef,
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
                // log::info!("add_clause({})", DisplaySlice(lits));

                solver.internal_backtrack(0);
                let res = solver.internal_propagate();
                assert!(res);

                let lits = clause_to_external(lits).collect_vec();
                if lits.len() >= 2 {
                    for lit in lits {
                        assert!(solver.is_active(lit), "lit {} is not active", lit);
                        solver.add_derived(lit);
                    }
                    solver.add_derived(0);
                } else {
                    let lit = lits[0];
                    if solver.is_active(lit) {
                        solver.add_unit_clause(lit);
                        assert!(!solver.is_active(lit));

                        // p3 -- CvK_10x10 -- unit + 2bin -- no-derive

                        // let var = lit.abs();
                        // for other in 1..solver.vars() {
                        //     let other = other as i32;
                        //     if other == var {
                        //         continue;
                        //     }
                        //     if solver.is_active(other) {
                        //         // log::warn!("Adding unit {} as two clauses: [{}, {}] and [{}, {}]", lit, lit, other, lit, -other);
                        //         solver.add_derived(lit);
                        //         solver.add_derived(other);
                        //         solver.add_derived(0);
                        //         solver.add_derived(lit);
                        //         solver.add_derived(-other);
                        //         solver.add_derived(0);
                        //         solver.add_unit_clause(lit);
                        //         break;
                        //     }
                        // }
                    } else {
                        log::warn!("unit {} is not active", lit);
                    }
                }

                let res = solver.internal_propagate();
                assert!(res);
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
                solver.propcheck_all_tree(&vars_external, limit, false)
                // solver.propcheck_all_tree_via_internal(&vars_external, limit, None)
            }
        }
    }
}
