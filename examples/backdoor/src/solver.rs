use cadical::statik::Cadical;
use cadical::{FixedResponse, SolveResponse};
use simple_sat::lit::Lit;
use simple_sat::var::Var;

use crate::utils::clause_to_external;

#[derive(Debug)]
pub struct Solver(pub Cadical);

impl Solver {
    pub fn new(cadical: Cadical) -> Self {
        Self(cadical)
    }
}

impl Solver {
    pub fn num_vars(&self) -> u64 {
        self.0.vars() as u64
    }

    pub fn is_already_assigned(&self, var: Var) -> bool {
        let lit = var.to_external() as i32;
        self.0.fixed(lit).unwrap() != FixedResponse::Unclear
    }

    pub fn is_active(&self, var: Var) -> bool {
        let lit = var.to_external() as i32;
        self.0.is_active(lit)
    }

    pub fn assume(&self, lit: Lit) {
        self.0.assume(lit.to_external()).unwrap();
    }

    pub fn add_clause(&self, lits: &[Lit]) {
        self.0.add_clause(clause_to_external(lits));
        // solver.add_derived_clause(clause_to_external(lits));
    }

    pub fn solve(&mut self) -> SolveResponse {
        self.0.solve().unwrap()
    }

    pub fn failed(&self, lit: Lit) -> bool {
        self.0.failed(lit.to_external()).unwrap()
    }

    pub fn propcheck(&self, lits: &[Lit]) -> (bool, u64) {
        let lits_external: Vec<i32> = lits.iter().map(|lit| lit.to_external()).collect();
        self.0.propcheck(&lits_external, false, false, false)
    }

    pub fn propcheck_save_core(&self, lits: &[Lit]) -> (bool, u64) {
        let lits_external: Vec<i32> = lits.iter().map(|lit| lit.to_external()).collect();
        self.0.propcheck(&lits_external, false, false, true)
    }

    pub fn propcheck_all_tree(&self, vars: &[Var], limit: u64) -> u64 {
        let vars_external: Vec<i32> = vars.iter().map(|var| var.to_external() as i32).collect();
        // self.0.propcheck_all_tree(&vars_external, limit, false)
        self.0.propcheck_all_tree_via_internal(&vars_external, limit, None, None)
    }
}
