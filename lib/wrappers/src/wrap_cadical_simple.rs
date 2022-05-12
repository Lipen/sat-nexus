use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use cadical::Cadical;
use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SimpleSolver, SolveResponse};

pub struct CadicalSimpleSolver {
    inner: Cadical,
    nvars: usize,
    nclauses: usize,
}

impl CadicalSimpleSolver {
    pub fn new() -> Self {
        Self::new_custom(Cadical::new())
    }

    pub fn new_custom(inner: Cadical) -> Self {
        Self {
            inner,
            nvars: 0,
            nclauses: 0,
        }
    }
}

impl Default for CadicalSimpleSolver {
    fn default() -> Self {
        CadicalSimpleSolver::new()
    }
}

impl From<Cadical> for CadicalSimpleSolver {
    fn from(inner: Cadical) -> Self {
        CadicalSimpleSolver::new_custom(inner)
    }
}

impl Display for CadicalSimpleSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CadicalSolver({})", self.inner)
    }
}

impl SimpleSolver for CadicalSimpleSolver {
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
        self.nvars
    }
    fn num_clauses(&self) -> usize {
        self.nclauses
    }

    fn new_var(&mut self) -> Lit {
        self.nvars += 1;
        Lit::new(self.nvars as i32)
    }

    fn assume(&mut self, lit: Lit) {
        self.inner.assume(lit.into());
    }

    fn add_clause(&mut self, lits: &[Lit]) {
        self.nclauses += 1;
        self.inner.add_clause(lits.iter().copied());
    }

    fn solve(&mut self) -> SolveResponse {
        use cadical::SolveResponse as CadicalSolveResponse;
        match self.inner.solve() {
            Ok(CadicalSolveResponse::Sat) => SolveResponse::Sat,
            Ok(CadicalSolveResponse::Unsat) => SolveResponse::Unsat,
            Ok(CadicalSolveResponse::Interrupted) => SolveResponse::Unknown,
            Err(e) => panic!("Could not solve: {}", e),
        }
    }

    fn value(&self, lit: Lit) -> LitValue {
        use cadical::LitValue as CadicalLitValue;
        match self.inner.val(lit.into()) {
            Ok(CadicalLitValue::True) => LitValue::True,
            Ok(CadicalLitValue::False) => LitValue::False,
            Err(e) => panic!("Could not get literal value: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_cadical() -> color_eyre::Result<()> {
        let mut solver = CadicalSimpleSolver::new();
        assert!(solver.signature().contains("cadical"));

        // Initializing variables
        let a = solver.new_var();
        let b = solver.new_var();
        let c = solver.new_var();
        let d = solver.new_var();
        assert_eq!(4, solver.num_vars());

        // Adding [(a or b) and (c or d) and not(a and b) and not(c and d)]
        solver.add_clause(&[a, b]);
        solver.add_clause(&[c, d]);
        solver.add_clause(&[-a, -b]);
        solver.add_clause(&[-c, -d]);

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
