use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;
use tap::Pipe;

use minisat::dynamic::Lit as MiniSatLit;
use minisat::dynamic::{LBool, MiniSat};
use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};

pub struct MiniSatDynamicSolver {
    inner: MiniSat,
    assumptions: Vec<MiniSatLit>,
}

impl MiniSatDynamicSolver {
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

impl Default for MiniSatDynamicSolver {
    fn default() -> Self {
        MiniSatDynamicSolver::new()
    }
}

impl From<MiniSat> for MiniSatDynamicSolver {
    fn from(inner: MiniSat) -> Self {
        MiniSatDynamicSolver::new_custom(inner)
    }
}

impl Debug for MiniSatDynamicSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiniSatSolver").field("inner", &self.inner).finish()
    }
}

impl Display for MiniSatDynamicSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", tynm::type_name::<Self>(), self.inner)
    }
}

impl Solver for MiniSatDynamicSolver {
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

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.assumptions.push(lit.into().pipe(to_ms));
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

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        match self.inner.model_value_lit(lit.into().pipe(to_ms)) {
            LBool::True => LitValue::True,
            LBool::False => LitValue::False,
            LBool::Undef => panic!("model_value_lit returned Undef"),
        }
    }
}

fn to_ms(lit: Lit) -> MiniSatLit {
    let lit = lit.get();
    debug_assert_ne!(lit, 0, "Literal must be non-zero");
    let var = (lit.abs() - 1) as _; // 0-based variable index
    let sign = if lit > 0 { 0 } else { 1 }; // 0 if positive, 1 if negative
    MiniSatLit::mk(var, sign)
}

fn from_ms(lit: MiniSatLit) -> Lit {
    let var = lit.var() + 1; // 1-based variable index
    let sign = lit.sign(); // 0 if positive, 1 if negative
    let lit = var as _;
    match sign {
        0 => Lit::new(lit),
        1 => Lit::new(-lit),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_minisat_dynamic() -> color_eyre::Result<()> {
        let mut solver = MiniSatDynamicSolver::new();
        assert!(solver.signature().contains("minisat"));

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
