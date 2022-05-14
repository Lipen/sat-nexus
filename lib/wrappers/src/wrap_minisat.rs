use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use itertools::Itertools;
use tap::Pipe;

use minisat::dynamic::Lit as MiniSatLit;
use minisat::dynamic::{LBool, MiniSat};
use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{BaseSolver, LitValue, SolveResponse, Solver};

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
        write!(f, "{}({})", tynm::type_name::<Self>(), self.signature())
    }
}

impl BaseSolver for MiniSatSolver {
    fn assume_(&mut self, lit: Lit) {
        self.assumptions.push(lit.pipe(to_ms));
    }

    fn value_(&self, lit: Lit) -> LitValue {
        match self.inner.model_value_lit(lit.pipe(to_ms)) {
            LBool::True => LitValue::True,
            LBool::False => LitValue::False,
            LBool::Undef => panic!("model_value_lit returned Undef"),
        }
    }

    fn add_clause_(&mut self, lits: &[Lit]) {
        self.inner.add_clause(lits.iter().copied().map(to_ms));
    }

    fn add_clause__(&mut self, lits: &mut dyn Iterator<Item = Lit>) {
        self.inner.add_clause(lits.map(to_ms));
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
        self.inner.new_lit().pipe(from_ms)
    }

    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        self.inner.add_clause(lits.into_iter().map_into::<Lit>().map(to_ms));
    }

    fn solve(&mut self) -> SolveResponse {
        // let assumptions = std::mem::replace(&mut self.assumptions, Vec::new());
        // match self.inner.solve_under_assumptions(assumptions) {
        match self.inner.solve_under_assumptions(self.assumptions.drain(..)) {
            true => SolveResponse::Sat,
            false => SolveResponse::Unsat,
        }
    }
}

fn to_ms(lit: Lit) -> MiniSatLit {
    let lit: i32 = lit.into();
    let var = lit.abs() - 1; // 0-based variable index
    let sign = if lit > 0 { 0 } else { 1 }; // 0 if positive, 1 if negative
    (2 * var + sign).into()
}

fn from_ms(lit: MiniSatLit) -> Lit {
    let lit: i32 = lit.into();
    let var = (lit >> 1) + 1; // 1-based variable index
    let sign = lit & 1; // 0 if negative, 1 if positive
    if sign > 0 {
        Lit::new(var)
    } else {
        Lit::new(-var)
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
        solver.add_clause([a, b]);
        solver.add_clause(&[c, d]);
        solver.add_clause(vec![-a, -b]);
        solver.add_clause(&vec![-c, -d]);

        // Problem is satisfiable
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        // Assuming both a and b to be true
        solver.assume(a);
        solver.assume(b);
        // Problem is unsatisfiable under assumptions
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Unsat);

        // `solve` resets assumptions, so calling it again should produce SAT
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        Ok(())
    }
}
