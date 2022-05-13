use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use strum::IntoStaticStr;

use sat_nexus_core::lit::Lit;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};

use crate::cadical::CadicalSolver;
use crate::minisat::MiniSatSolver;

#[derive(IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum DispatchingSolver {
    MiniSat(MiniSatSolver),
    Cadical(CadicalSolver),
}

impl DispatchingSolver {
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

impl Display for DispatchingSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        match self {
            DispatchingSolver::MiniSat(inner) => write!(f, "DispatchingSolver::{}({})", name, inner),
            DispatchingSolver::Cadical(inner) => write!(f, "DispatchingSolver::{}({})", name, inner),
        }
    }
}

impl Solver for DispatchingSolver {
    fn signature(&self) -> Cow<str> {
        match self {
            DispatchingSolver::MiniSat(inner) => inner.signature(),
            DispatchingSolver::Cadical(inner) => inner.signature(),
        }
    }

    fn reset(&mut self) {
        match self {
            DispatchingSolver::MiniSat(inner) => inner.reset(),
            DispatchingSolver::Cadical(inner) => inner.reset(),
        }
    }

    fn release(&mut self) {
        match self {
            DispatchingSolver::MiniSat(inner) => inner.release(),
            DispatchingSolver::Cadical(inner) => inner.release(),
        }
    }

    fn num_vars(&self) -> usize {
        match self {
            DispatchingSolver::MiniSat(inner) => inner.num_vars(),
            DispatchingSolver::Cadical(inner) => inner.num_vars(),
        }
    }

    fn num_clauses(&self) -> usize {
        match self {
            DispatchingSolver::MiniSat(inner) => inner.num_clauses(),
            DispatchingSolver::Cadical(inner) => inner.num_clauses(),
        }
    }

    fn new_var(&mut self) -> Lit {
        match self {
            DispatchingSolver::MiniSat(inner) => inner.new_var(),
            DispatchingSolver::Cadical(inner) => inner.new_var(),
        }
    }

    fn assume_(&mut self, lit: Lit) {
        match self {
            DispatchingSolver::MiniSat(inner) => inner.assume_(lit),
            DispatchingSolver::Cadical(inner) => inner.assume_(lit),
        }
    }

    fn add_clause_(&mut self, lits: &[Lit]) {
        match self {
            DispatchingSolver::MiniSat(inner) => inner.add_clause_(lits),
            DispatchingSolver::Cadical(inner) => inner.add_clause_(lits),
        }
    }

    fn solve(&mut self) -> SolveResponse {
        match self {
            DispatchingSolver::MiniSat(inner) => inner.solve(),
            DispatchingSolver::Cadical(inner) => inner.solve(),
        }
    }

    fn value_(&self, lit: Lit) -> LitValue {
        match self {
            DispatchingSolver::MiniSat(inner) => inner.value_(lit),
            DispatchingSolver::Cadical(inner) => inner.value_(lit),
        }
    }
}
