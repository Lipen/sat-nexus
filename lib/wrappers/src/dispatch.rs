use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use strum::IntoStaticStr;

use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::delegate::DelegateSolver;
use sat_nexus_core::solver::simple::SimpleSolver;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};

use crate::cadical_dynamic::CadicalDynamicSolver;
use crate::kissat_dynamic::KissatDynamicSolver;
use crate::minisat_dynamic::MiniSatDynamicSolver;

#[derive(Debug, IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum DispatchSolver {
    Delegate(DelegateSolver),
    MiniSatDynamic(MiniSatDynamicSolver),
    CadicalDynamic(CadicalDynamicSolver),
    KissatDynamic(KissatDynamicSolver),
}

macro_rules! dispatch {
    ($value:expr, $pattern:pat => $result:expr) => {
        match $value {
            DispatchSolver::Delegate($pattern) => $result,
            DispatchSolver::MiniSatDynamic($pattern) => $result,
            DispatchSolver::CadicalDynamic($pattern) => $result,
            DispatchSolver::KissatDynamic($pattern) => $result,
        }
    };
}

impl DispatchSolver {
    pub fn new_delegate(solver: impl SimpleSolver + 'static) -> Self {
        Self::from(DelegateSolver::new(solver))
    }
    pub fn new_delegate_wrap(solver: impl Solver + 'static) -> Self {
        Self::from(DelegateSolver::wrap(solver))
    }
    pub fn new_minisat() -> Self {
        Self::from(MiniSatDynamicSolver::new())
    }
    pub fn new_cadical() -> Self {
        Self::from(CadicalDynamicSolver::new())
    }
    pub fn new_kissat() -> Self {
        Self::from(KissatDynamicSolver::new())
    }

    pub fn by_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "minisat" => Self::new_minisat(),
            "cadical" => Self::new_cadical(),
            "kissat" => Self::new_kissat(),
            _ => panic!("Bad name '{}'", name),
        }
    }
}

impl From<DelegateSolver> for DispatchSolver {
    fn from(inner: DelegateSolver) -> Self {
        DispatchSolver::Delegate(inner)
    }
}

impl From<MiniSatDynamicSolver> for DispatchSolver {
    fn from(inner: MiniSatDynamicSolver) -> Self {
        DispatchSolver::MiniSatDynamic(inner)
    }
}

impl From<CadicalDynamicSolver> for DispatchSolver {
    fn from(inner: CadicalDynamicSolver) -> Self {
        DispatchSolver::CadicalDynamic(inner)
    }
}

impl From<KissatDynamicSolver> for DispatchSolver {
    fn from(inner: KissatDynamicSolver) -> Self {
        DispatchSolver::KissatDynamic(inner)
    }
}

impl Display for DispatchSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        dispatch! { self, inner =>
            write!(f, "{}::{}({})", tynm::type_name::<Self>(), name, inner)
        }
    }
}

macro_rules! dispatch_delegate {
    ($value:ident, $name:ident($($args:tt)*)) => {
        dispatch!($value, inner => inner.$name($($args)*))
    };
}

impl Solver for DispatchSolver {
    fn signature(&self) -> Cow<str> {
        dispatch_delegate!(self, signature())
    }

    fn reset(&mut self) {
        dispatch_delegate!(self, reset())
    }

    fn release(&mut self) {
        dispatch_delegate!(self, release())
    }

    fn num_vars(&self) -> usize {
        dispatch_delegate!(self, num_vars())
    }

    fn num_clauses(&self) -> usize {
        dispatch_delegate!(self, num_clauses())
    }

    fn new_var(&mut self) -> Lit {
        dispatch_delegate!(self, new_var())
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        dispatch_delegate!(self, assume(lit))
    }

    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        dispatch_delegate!(self, add_clause(lits))
    }

    fn add_clause_<L>(&mut self, lits: &[L])
    where
        L: Into<Lit> + Copy,
    {
        dispatch_delegate!(self, add_clause_(lits))
    }

    fn add_unit<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        dispatch_delegate!(self, add_unit(lit))
    }

    fn solve(&mut self) -> SolveResponse {
        dispatch_delegate!(self, solve())
    }

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        dispatch_delegate!(self, value(lit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cadical_dynamic::CadicalDynamicSolver;

    fn run_test(mut solver: DispatchSolver, use_assumptions: bool) -> color_eyre::Result<()> {
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

        if use_assumptions {
            // Assuming both a and b to be true
            solver.assume(a);
            solver.assume(b);
            // Problem is unsatisfiable under assumptions
            let response = solver.solve();
            assert_eq!(response, SolveResponse::Unsat);

            // `solve` resets assumptions, so calling it again should produce SAT
            let response = solver.solve();
            assert_eq!(response, SolveResponse::Sat);
        }

        Ok(())
    }

    #[test]
    fn test_dispatch_delegate_minisat() -> color_eyre::Result<()> {
        let solver = DispatchSolver::new_delegate_wrap(MiniSatDynamicSolver::new());
        assert!(matches!(solver, DispatchSolver::Delegate(_)));
        assert!(solver.signature().contains("minisat"));
        run_test(solver, true)
    }

    #[test]
    fn test_dispatch_delegate_cadical() -> color_eyre::Result<()> {
        let solver = DispatchSolver::new_delegate_wrap(CadicalDynamicSolver::new());
        assert!(matches!(solver, DispatchSolver::Delegate(_)));
        assert!(solver.signature().contains("cadical"));
        run_test(solver, true)
    }

    #[test]
    fn test_dispatch_delegate_kissat() -> color_eyre::Result<()> {
        let solver = DispatchSolver::new_delegate_wrap(KissatDynamicSolver::new());
        assert!(matches!(solver, DispatchSolver::Delegate(_)));
        assert!(solver.signature().contains("kissat"));
        run_test(solver, false)
    }

    #[test]
    fn test_dispatch_minisat() -> color_eyre::Result<()> {
        let solver = DispatchSolver::new_minisat();
        assert!(matches!(solver, DispatchSolver::MiniSatDynamic(_)));
        assert!(solver.signature().contains("minisat"));
        run_test(solver, true)
    }

    #[test]
    fn test_dispatch_cadical() -> color_eyre::Result<()> {
        let solver = DispatchSolver::new_cadical();
        assert!(matches!(solver, DispatchSolver::CadicalDynamic(_)));
        assert!(solver.signature().contains("cadical"));
        run_test(solver, true)
    }

    #[test]
    fn test_dispatch_kissat() -> color_eyre::Result<()> {
        let solver = DispatchSolver::new_kissat();
        assert!(matches!(solver, DispatchSolver::KissatDynamic(_)));
        assert!(solver.signature().contains("kissat"));
        run_test(solver, false)
    }
}
