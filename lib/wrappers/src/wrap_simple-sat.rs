use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;
use tap::Pipe;

use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};
use simple_sat::lit::Lit as SimpleSatLit;
use simple_sat::solver::Solver as SimpleSat;

pub struct SimpleSatSolver {
    inner: SimpleSat,
    assumptions: Vec<SimpleSatLit>,
}

impl SimpleSatSolver {
    pub fn new() -> Self {
        Self::new_custom(SimpleSat::default())
    }

    pub fn new_custom(inner: SimpleSat) -> Self {
        Self {
            inner,
            assumptions: Vec::new(),
        }
    }
}

impl Default for SimpleSatSolver {
    fn default() -> Self {
        SimpleSatSolver::new()
    }
}

impl From<SimpleSat> for SimpleSatSolver {
    fn from(inner: SimpleSat) -> Self {
        SimpleSatSolver::new_custom(inner)
    }
}

impl Debug for SimpleSatSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleSatSolver").field("inner", &self.inner).finish()
    }
}

impl Display for SimpleSatSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", tynm::type_name::<Self>(), self.inner)
    }
}

impl Solver for SimpleSatSolver {
    fn signature(&self) -> Cow<str> {
        "simple-sat".into()
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
    fn release(&mut self) {
        todo!()
        // self.inner.release();
    }

    fn num_vars(&self) -> usize {
        self.inner.num_vars()
    }
    fn num_clauses(&self) -> usize {
        self.inner.num_clauses()
    }

    fn new_var(&mut self) -> Lit {
        let var = self.inner.new_var();
        let lit = SimpleSatLit::new(var, false);
        from_ss(lit)
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.assumptions.push(lit.into().pipe(to_ss));
    }

    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        let lits = lits.into_iter().map_into::<Lit>().map(to_ss).collect_vec();
        self.inner.add_clause(&lits);
    }

    fn solve(&mut self) -> SolveResponse {
        use simple_sat::solver::SolveResult;
        let assumptions = std::mem::replace(&mut self.assumptions, Vec::new());
        match self.inner.solve_under_assumptions(&assumptions) {
            SolveResult::Sat => SolveResponse::Sat,
            SolveResult::Unsat => SolveResponse::Unsat,
            SolveResult::Unknown => SolveResponse::Unknown,
        }
    }

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        use simple_sat::lbool::LBool;
        match self.inner.value(lit.into().pipe(to_ss)) {
            LBool::True => LitValue::True,
            LBool::False => LitValue::False,
            LBool::Undef => panic!("value returned Undef"),
        }
    }
}

fn to_ss(lit: Lit) -> SimpleSatLit {
    let lit = lit.get();
    debug_assert_ne!(lit, 0, "Literal must be non-zero");
    SimpleSatLit::from_external(lit)
}

fn from_ss(lit: SimpleSatLit) -> Lit {
    let lit = lit.to_external();
    debug_assert_ne!(lit, 0, "Literal must be non-zero");
    Lit::from(lit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_simple_sat() -> color_eyre::Result<()> {
        let mut solver = SimpleSatSolver::new();
        assert!(solver.signature().contains("simple-sat"));

        // Initializing variables
        let a = solver.new_var();
        let b = solver.new_var();
        let c = solver.new_var();
        let d = solver.new_var();
        assert_eq!(solver.num_vars(), 4);
        assert_eq!(a.get(), 1);
        assert_eq!(b.get(), 2);
        assert_eq!(c.get(), 3);
        assert_eq!(d.get(), 4);

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
