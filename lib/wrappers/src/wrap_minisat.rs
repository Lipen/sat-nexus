use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use easy_ext::ext;
use itertools::Itertools;

use minisat::dynamic::Lit as MiniSatLit;
use minisat::dynamic::{LBool, MiniSat};
use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};

pub struct MiniSatSolver {
    inner: MiniSat,
    assumptions: Vec<MiniSatLit>,
}

impl MiniSatSolver {
    pub fn new() -> Self {
        Self::new_custom(MiniSat::new())
    }

    pub fn new_custom(inner: MiniSat) -> Self {
        Self {
            inner,
            assumptions: Vec::new(),
        }
    }
}

impl Default for MiniSatSolver {
    fn default() -> Self {
        MiniSatSolver::new()
    }
}

impl From<MiniSat> for MiniSatSolver {
    fn from(inner: MiniSat) -> Self {
        MiniSatSolver::new_custom(inner)
    }
}

impl Display for MiniSatSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: change to MiniSatSolver after global refactoring
        write!(f, "MiniSatSimpleSolver({})", self.signature())
    }
}

impl Solver for MiniSatSolver {
    fn signature(&self) -> Cow<str> {
        self.inner.signature().into()
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
    fn release(&mut self) {
        self.inner.release();
    }

    fn num_vars(&self) -> usize {
        self.inner.num_vars() as usize
    }
    fn num_clauses(&self) -> usize {
        self.inner.num_clauses() as usize
    }

    fn new_var(&mut self) -> Lit {
        self.inner.new_lit().to_lit()
    }

    fn assume_(&mut self, lit: Lit) {
        self.assumptions.push(lit.to_ms_lit());
    }

    fn add_clause_(&mut self, lits: &[Lit]) {
        // FIXME
        let lits = lits.iter().map_into::<Lit>().map(Lit::to_ms_lit);
        self.inner.add_clause(lits);
    }

    fn solve(&mut self) -> SolveResponse {
        // let assumptions = std::mem::replace(&mut self.assumptions, Vec::new());
        // match self.inner.solve_under_assumptions(assumptions) {
        match self.inner.solve_under_assumptions(self.assumptions.drain(..)) {
            true => SolveResponse::Sat,
            false => SolveResponse::Unsat,
        }
    }

    fn value_(&self, lit: Lit) -> LitValue {
        match self.inner.model_value_lit(lit.to_ms_lit()) {
            LBool::True => LitValue::True,
            LBool::False => LitValue::False,
            LBool::Undef => panic!("model_value_lit returned Undef"),
        }
    }
}

#[ext]
impl Lit {
    fn to_ms_lit(self) -> MiniSatLit {
        let lit: i32 = self.into();
        let var = lit.abs() - 1; // 0-based variable index
        let sign = if lit > 0 { 0 } else { 1 }; // 0 if positive, 1 if negative
        (2 * var + sign).into()
    }
}

#[ext]
impl MiniSatLit {
    fn to_lit(self) -> Lit {
        let lit: i32 = self.into();
        let var = (lit >> 1) + 1; // 1-based variable index
        let sign = lit & 1; // 0 if negative, 1 if positive
        if sign > 0 {
            Lit::new(var)
        } else {
            Lit::new(-var)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_minisat() -> color_eyre::Result<()> {
        let mut solver = MiniSatSolver::new();
        assert!(solver.signature().contains("minisat"));

        // Initializing variables
        let a = solver.new_var();
        let b = solver.new_var();
        let c = solver.new_var();
        let d = solver.new_var();
        assert_eq!(solver.num_vars(), 4);

        // Adding [(a or b) and (c or d) and not(a and b) and not(c and d)]
        solver.add_clause_(&[a, b]);
        solver.add_clause_(&[c, d]);
        solver.add_clause_(&[-a, -b]);
        solver.add_clause_(&[-c, -d]);

        // Problem is satisfiable
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        // Assuming both a and b to be true
        solver.assume_(a);
        solver.assume_(b);
        // Problem is unsatisfiable under assumptions
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Unsat);

        // `solve` resets assumptions, so calling it again should produce SAT
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        Ok(())
    }
}
