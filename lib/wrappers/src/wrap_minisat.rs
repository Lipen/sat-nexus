use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;

use minisat::{LBool, MiniSat};
use sat_nexus_core::context::Context;
use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};

pub struct WrappedMiniSat {
    inner: MiniSat,
    context: Rc<RefCell<Context>>,
    assumptions: Vec<minisat::Lit>,
}

impl WrappedMiniSat {
    pub fn new() -> Self {
        Self::new_custom(MiniSat::new())
    }

    pub fn new_custom(inner: MiniSat) -> Self {
        Self {
            inner,
            context: Rc::new(RefCell::new(Context::new())),
            assumptions: Vec::new(),
        }
    }
}

impl Default for WrappedMiniSat {
    fn default() -> Self {
        WrappedMiniSat::new()
    }
}

impl From<MiniSat> for WrappedMiniSat {
    fn from(inner: MiniSat) -> Self {
        WrappedMiniSat::new_custom(inner)
    }
}

impl fmt::Display for WrappedMiniSat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WrappedSolver({})", self.signature())
    }
}

impl Solver for WrappedMiniSat {
    fn signature(&self) -> Cow<str> {
        self.inner.signature().into()
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
    fn release(&mut self) {
        self.inner.release();
    }

    fn context(&self) -> Rc<RefCell<Context>> {
        Rc::clone(&self.context)
    }

    fn num_vars(&self) -> usize {
        self.inner.num_vars() as usize
    }
    fn num_clauses(&self) -> usize {
        self.inner.num_clauses() as usize
    }

    fn new_var(&mut self) -> Lit {
        from_ms_lit(self.inner.new_lit())
    }

    fn add_clause<I, L>(&mut self, lits: I)
    where
        I: IntoIterator<Item = L>,
        L: Into<Lit>,
    {
        let lits = lits.into_iter().map_into::<Lit>().map(to_ms_lit);
        self.inner.add_clause(lits);
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        self.assumptions.push(to_ms_lit(lit.into()));
    }

    fn solve(&mut self) -> SolveResponse {
        // let assumptions = std::mem::replace(&mut self.assumptions, Vec::new());
        // match self.inner.solve_under_assumptions(assumptions) {
        match self.inner.solve_under_assumptions(self.assumptions.drain(..)) {
            true => SolveResponse::Sat,
            false => SolveResponse::Unsat,
        }
    }

    fn val<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        match self.inner.model_value_lit(to_ms_lit(lit.into())) {
            LBool::True => LitValue::True,
            LBool::False => LitValue::False,
            LBool::Undef => panic!("model_value_lit returned Undef"),
        }
    }
}

fn to_ms_lit(lit: Lit) -> minisat::Lit {
    let lit: i32 = lit.into();
    let var = lit.abs() - 1; // 0-based variable index
    let sign = if lit > 0 { 1 } else { 0 }; // 0 if negative, 1 if positive
    minisat::Lit::from(2 * var + sign)
}

fn from_ms_lit(lit: minisat::Lit) -> Lit {
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
    fn test_wrap_solver() -> color_eyre::Result<()> {
        let mut solver = WrappedMiniSat::new();
        assert!(solver.signature().contains("cadical"));

        // Adding [(1 or 2) and (3 or 4) and not(1 and 2) and not(3 and 4)]
        solver.add_clause([1, 2]);
        solver.add_clause(&[3, 4]);
        solver.add_clause(vec![-1, -2]);
        solver.add_clause(&vec![-3, -4]);

        // Problem is satisfiable
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        // Assuming both 1 and 2 to be true
        solver.assume(1);
        solver.assume(&2);
        // Problem is unsatisfiable under assumptions
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Unsat);

        // `solve` resets assumptions, so calling it again should produce SAT
        let response = solver.solve();
        assert_eq!(response, SolveResponse::Sat);

        Ok(())
    }
}
