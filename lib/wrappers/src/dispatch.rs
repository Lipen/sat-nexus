use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use strum::IntoStaticStr;

use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::delegate::DelegateSolver;
use sat_nexus_core::solver::{BaseSolver, LitValue, SolveResponse, Solver};

use crate::cadical::CadicalSolver;
use crate::minisat::MiniSatSolver;

#[derive(IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum DispatchingSolver {
    Delegate(DelegateSolver),
    MiniSat(MiniSatSolver),
    Cadical(CadicalSolver),
}

impl DispatchingSolver {
    pub fn new_delegate(solver: impl Solver + 'static) -> Self {
        DispatchingSolver::Delegate(DelegateSolver::new(solver))
    }
    pub fn new_minisat() -> Self {
        DispatchingSolver::MiniSat(MiniSatSolver::new())
    }
    pub fn new_cadical() -> Self {
        DispatchingSolver::Cadical(CadicalSolver::new())
    }

    pub fn by_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "minisat" => Self::new_minisat(),
            "cadical" => Self::new_cadical(),
            _ => panic!("Bad name '{}'", name),
        }
    }
}

impl From<DelegateSolver> for DispatchingSolver {
    fn from(inner: DelegateSolver) -> Self {
        DispatchingSolver::Delegate(inner)
    }
}
impl From<MiniSatSolver> for DispatchingSolver {
    fn from(inner: MiniSatSolver) -> Self {
        DispatchingSolver::MiniSat(inner)
    }
}
impl From<CadicalSolver> for DispatchingSolver {
    fn from(inner: CadicalSolver) -> Self {
        DispatchingSolver::Cadical(inner)
    }
}

impl Display for DispatchingSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        match self {
            DispatchingSolver::Delegate(inner) => {
                write!(f, "{}::{}({})", tynm::type_name::<Self>(), name, inner)
            }
            DispatchingSolver::MiniSat(inner) => {
                write!(f, "{}::{}({})", tynm::type_name::<Self>(), name, inner)
            }
            DispatchingSolver::Cadical(inner) => {
                write!(f, "{}::{}({})", tynm::type_name::<Self>(), name, inner)
            }
        }
    }
}

impl BaseSolver for DispatchingSolver {
    fn assume_(&mut self, lit: Lit) {
        match self {
            DispatchingSolver::Delegate(inner) => inner.assume_(lit),
            DispatchingSolver::MiniSat(inner) => inner.assume_(lit),
            DispatchingSolver::Cadical(inner) => inner.assume_(lit),
        }
    }

    fn value_(&self, lit: Lit) -> LitValue {
        match self {
            DispatchingSolver::Delegate(inner) => inner.value_(lit),
            DispatchingSolver::MiniSat(inner) => inner.value_(lit),
            DispatchingSolver::Cadical(inner) => inner.value_(lit),
        }
    }

    fn add_clause_(&mut self, lits: &[Lit]) {
        match self {
            DispatchingSolver::Delegate(inner) => inner.add_clause_(lits),
            DispatchingSolver::MiniSat(inner) => inner.add_clause_(lits),
            DispatchingSolver::Cadical(inner) => inner.add_clause_(lits),
        }
    }

    fn add_clause__(&mut self, lits: &mut dyn Iterator<Item = Lit>) {
        match self {
            DispatchingSolver::Delegate(inner) => inner.add_clause__(lits),
            DispatchingSolver::MiniSat(inner) => inner.add_clause__(lits),
            DispatchingSolver::Cadical(inner) => inner.add_clause__(lits),
        }
    }
}

impl Solver for DispatchingSolver {
    fn signature(&self) -> Cow<str> {
        match self {
            DispatchingSolver::Delegate(inner) => inner.signature(),
            DispatchingSolver::MiniSat(inner) => inner.signature(),
            DispatchingSolver::Cadical(inner) => inner.signature(),
        }
    }

    fn reset(&mut self) {
        match self {
            DispatchingSolver::Delegate(inner) => inner.reset(),
            DispatchingSolver::MiniSat(inner) => inner.reset(),
            DispatchingSolver::Cadical(inner) => inner.reset(),
        }
    }

    fn release(&mut self) {
        match self {
            DispatchingSolver::Delegate(inner) => inner.release(),
            DispatchingSolver::MiniSat(inner) => inner.release(),
            DispatchingSolver::Cadical(inner) => inner.release(),
        }
    }

    fn num_vars(&self) -> usize {
        match self {
            DispatchingSolver::Delegate(inner) => inner.num_vars(),
            DispatchingSolver::MiniSat(inner) => inner.num_vars(),
            DispatchingSolver::Cadical(inner) => inner.num_vars(),
        }
    }

    fn num_clauses(&self) -> usize {
        match self {
            DispatchingSolver::Delegate(inner) => inner.num_clauses(),
            DispatchingSolver::MiniSat(inner) => inner.num_clauses(),
            DispatchingSolver::Cadical(inner) => inner.num_clauses(),
        }
    }

    fn new_var(&mut self) -> Lit {
        match self {
            DispatchingSolver::Delegate(inner) => inner.new_var(),
            DispatchingSolver::MiniSat(inner) => inner.new_var(),
            DispatchingSolver::Cadical(inner) => inner.new_var(),
        }
    }

    fn assume<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        match self {
            DispatchingSolver::Delegate(inner) => inner.assume(lit),
            DispatchingSolver::MiniSat(inner) => inner.assume(lit),
            DispatchingSolver::Cadical(inner) => inner.assume(lit),
        }
    }

    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        match self {
            DispatchingSolver::Delegate(inner) => inner.add_clause(lits),
            DispatchingSolver::MiniSat(inner) => inner.add_clause(lits),
            DispatchingSolver::Cadical(inner) => inner.add_clause(lits),
        }
    }

    fn add_unit<L>(&mut self, lit: L)
    where
        L: Into<Lit>,
    {
        match self {
            DispatchingSolver::Delegate(inner) => inner.add_unit(lit),
            DispatchingSolver::MiniSat(inner) => inner.add_unit(lit),
            DispatchingSolver::Cadical(inner) => inner.add_unit(lit),
        }
    }

    fn solve(&mut self) -> SolveResponse {
        match self {
            DispatchingSolver::Delegate(inner) => inner.solve(),
            DispatchingSolver::MiniSat(inner) => inner.solve(),
            DispatchingSolver::Cadical(inner) => inner.solve(),
        }
    }

    fn value<L>(&self, lit: L) -> LitValue
    where
        L: Into<Lit>,
    {
        match self {
            DispatchingSolver::Delegate(inner) => inner.value(lit),
            DispatchingSolver::MiniSat(inner) => inner.value(lit),
            DispatchingSolver::Cadical(inner) => inner.value(lit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_test(mut solver: DispatchingSolver) -> color_eyre::Result<()> {
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
        let solver = DispatchingSolver::new_delegate(MiniSatSolver::new());
        assert!(matches!(solver, DispatchingSolver::Delegate(_)));
        assert!(solver.signature().contains("minisat"));
        run_test(solver)
    }
    #[test]
    fn test_dispatch_delegate_cadical() -> color_eyre::Result<()> {
        let solver = DispatchingSolver::new_delegate(CadicalSolver::new());
        assert!(matches!(solver, DispatchingSolver::Delegate(_)));
        assert!(solver.signature().contains("cadical"));
        run_test(solver)
    }

    #[test]
    fn test_dispatch_minisat() -> color_eyre::Result<()> {
        let solver = DispatchingSolver::new_minisat();
        assert!(matches!(solver, DispatchingSolver::MiniSat(_)));
        assert!(solver.signature().contains("minisat"));
        run_test(solver)
    }
    #[test]
    fn test_dispatch_cadical() -> color_eyre::Result<()> {
        let solver = DispatchingSolver::new_cadical();
        assert!(matches!(solver, DispatchingSolver::Cadical(_)));
        assert!(solver.signature().contains("cadical"));
        run_test(solver)
    }
}
