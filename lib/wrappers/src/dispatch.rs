use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use strum::IntoStaticStr;

use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::delegate::DelegateSolver;
use sat_nexus_core::solver::simple::SimpleSolver;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};

use crate::cadical::CadicalSolver;
use crate::minisat::MiniSatSolver;

#[derive(IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum DispatchSolver {
    Delegate(DelegateSolver),
    MiniSat(MiniSatSolver),
    Cadical(CadicalSolver),
}

impl DispatchSolver {
    pub fn new_delegate(solver: impl SimpleSolver + 'static) -> Self {
        Self::from(DelegateSolver::new(solver))
    }
    pub fn new_delegate_wrap(solver: impl Solver + 'static) -> Self {
        Self::from(DelegateSolver::wrap(solver))
    }
    pub fn new_minisat() -> Self {
        Self::from(MiniSatSolver::new())
    }
    pub fn new_cadical() -> Self {
        Self::from(CadicalSolver::new())
    }

    pub fn by_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "minisat" => Self::new_minisat(),
            "cadical" => Self::new_cadical(),
            _ => panic!("Bad name '{}'", name),
        }
    }
}

impl From<DelegateSolver> for DispatchSolver {
    fn from(inner: DelegateSolver) -> Self {
        DispatchSolver::Delegate(inner)
    }
}
impl From<MiniSatSolver> for DispatchSolver {
    fn from(inner: MiniSatSolver) -> Self {
        DispatchSolver::MiniSat(inner)
    }
}
impl From<CadicalSolver> for DispatchSolver {
    fn from(inner: CadicalSolver) -> Self {
        DispatchSolver::Cadical(inner)
    }
}

impl Display for DispatchSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        match self {
            DispatchSolver::Delegate(inner) => {
                write!(f, "{}::{}({})", tynm::type_name::<Self>(), name, inner)
            }
            DispatchSolver::MiniSat(inner) => {
                write!(f, "{}::{}({})", tynm::type_name::<Self>(), name, inner)
            }
            DispatchSolver::Cadical(inner) => {
                write!(f, "{}::{}({})", tynm::type_name::<Self>(), name, inner)
            }
        }
    }
}

impl Solver for DispatchSolver {
    fn signature(&self) -> Cow<str> {
        match self {
            DispatchSolver::Delegate(inner) => inner.signature(),
            DispatchSolver::MiniSat(inner) => inner.signature(),
            DispatchSolver::Cadical(inner) => inner.signature(),
        }
    }

    fn reset(&mut self) {
        match self {
            DispatchSolver::Delegate(inner) => inner.reset(),
            DispatchSolver::MiniSat(inner) => inner.reset(),
            DispatchSolver::Cadical(inner) => inner.reset(),
        }
    }

    fn release(&mut self) {
        match self {
            DispatchSolver::Delegate(inner) => inner.release(),
            DispatchSolver::MiniSat(inner) => inner.release(),
            DispatchSolver::Cadical(inner) => inner.release(),
        }
    }

    fn num_vars(&self) -> usize {
        match self {
            DispatchSolver::Delegate(inner) => inner.num_vars(),
            DispatchSolver::MiniSat(inner) => inner.num_vars(),
            DispatchSolver::Cadical(inner) => inner.num_vars(),
        }
    }

    fn num_clauses(&self) -> usize {
        match self {
            DispatchSolver::Delegate(inner) => inner.num_clauses(),
            DispatchSolver::MiniSat(inner) => inner.num_clauses(),
            DispatchSolver::Cadical(inner) => inner.num_clauses(),
        }
    }

    fn new_var(&mut self) -> Lit {
        match self {
            DispatchSolver::Delegate(inner) => inner.new_var(),
            DispatchSolver::MiniSat(inner) => inner.new_var(),
            DispatchSolver::Cadical(inner) => inner.new_var(),
        }
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        match self {
            DispatchSolver::Delegate(inner) => inner.assume(lit),
            DispatchSolver::MiniSat(inner) => inner.assume(lit),
            DispatchSolver::Cadical(inner) => inner.assume(lit),
        }
    }

    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        match self {
            DispatchSolver::Delegate(inner) => inner.add_clause(lits),
            DispatchSolver::MiniSat(inner) => inner.add_clause(lits),
            DispatchSolver::Cadical(inner) => inner.add_clause(lits),
        }
    }

    fn add_unit<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        match self {
            DispatchSolver::Delegate(inner) => inner.add_unit(lit),
            DispatchSolver::MiniSat(inner) => inner.add_unit(lit),
            DispatchSolver::Cadical(inner) => inner.add_unit(lit),
        }
    }

    fn solve(&mut self) -> SolveResponse {
        match self {
            DispatchSolver::Delegate(inner) => inner.solve(),
            DispatchSolver::MiniSat(inner) => inner.solve(),
            DispatchSolver::Cadical(inner) => inner.solve(),
        }
    }

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        match self {
            DispatchSolver::Delegate(inner) => inner.value(lit),
            DispatchSolver::MiniSat(inner) => inner.value(lit),
            DispatchSolver::Cadical(inner) => inner.value(lit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_test(mut solver: DispatchSolver) -> color_eyre::Result<()> {
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

    #[test]
    fn test_dispatch_delegate_minisat() -> color_eyre::Result<()> {
        let solver = DispatchSolver::new_delegate_wrap(MiniSatSolver::new());
        assert!(matches!(solver, DispatchSolver::Delegate(_)));
        assert!(solver.signature().contains("minisat"));
        run_test(solver)
    }
    #[test]
    fn test_dispatch_delegate_cadical() -> color_eyre::Result<()> {
        let solver = DispatchSolver::new_delegate_wrap(CadicalSolver::new());
        assert!(matches!(solver, DispatchSolver::Delegate(_)));
        assert!(solver.signature().contains("cadical"));
        run_test(solver)
    }

    #[test]
    fn test_dispatch_minisat() -> color_eyre::Result<()> {
        let solver = DispatchSolver::new_minisat();
        assert!(matches!(solver, DispatchSolver::MiniSat(_)));
        assert!(solver.signature().contains("minisat"));
        run_test(solver)
    }
    #[test]
    fn test_dispatch_cadical() -> color_eyre::Result<()> {
        let solver = DispatchSolver::new_cadical();
        assert!(matches!(solver, DispatchSolver::Cadical(_)));
        assert!(solver.signature().contains("cadical"));
        run_test(solver)
    }
}
