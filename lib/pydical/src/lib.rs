use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use cadical::statik::{Cadical, CadicalError};
use cadical::SolveResponse;

#[pymodule]
fn pydical(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pydical>()?;
    Ok(())
}

#[pyclass(str)]
struct Pydical {
    cadical: Cadical,
}

unsafe impl Send for Pydical {}
unsafe impl Sync for Pydical {}

impl Display for Pydical {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pydical({})", self.cadical)
    }
}

#[pymethods]
impl Pydical {
    #[new]
    pub fn new() -> Self {
        Self { cadical: Cadical::new() }
    }

    pub fn signature(&self) -> &'static str {
        self.cadical.signature()
    }

    pub fn release(&mut self) {
        self.cadical.release();
    }

    pub fn reset(&mut self) {
        self.cadical.reset();
    }

    pub fn limit(&self, name: &str, limit: i32) {
        self.cadical.limit(name, limit);
    }

    pub fn add(&self, lit: i32) {
        self.cadical.add(lit);
    }

    #[pyo3(signature = (*lits))]
    pub fn add_clause(&self, lits: Vec<i32>) {
        self.cadical.add_clause(lits);
    }

    pub fn read_dimacs(&self, path: PathBuf) {
        self.cadical.read_dimacs(path, 0);
    }

    pub fn write_dimacs(&self, path: PathBuf) {
        self.cadical.write_dimacs(path);
    }

    pub fn assume(&self, lit: i32) -> Result<(), MyCadicalError> {
        self.cadical.assume(lit)?;
        Ok(())
    }

    /// Returns `True` for SAT.
    /// Returns `False` for UNSAT.
    /// Returns `None` for UNKNOWN.
    #[pyo3(signature = (assumptions=vec![]), text_signature = "(assumptions=[])")]
    pub fn solve(&mut self, assumptions: Vec<i32>) -> Result<Option<bool>, MyCadicalError> {
        for lit in assumptions {
            self.cadical.assume(lit)?;
        }
        let res = self.cadical.solve()?;
        Ok(match res {
            SolveResponse::Sat => Some(true),
            SolveResponse::Unsat => Some(false),
            SolveResponse::Interrupted => None,
        })
    }

    pub fn val(&self, lit: i32) -> Result<bool, MyCadicalError> {
        let res = self.cadical.val(lit).map(|v| v.into())?;
        Ok(res)
    }

    pub fn failed(&self, lit: i32) -> Result<bool, MyCadicalError> {
        let res = self.cadical.failed(lit).map(|v| v)?;
        Ok(res)
    }

    /// Returns `1` if `lit` is implied by the formula.
    /// Returns `-1` if `-lit` is implied by the formula.
    /// Returns `0` if it is unclear whether the literal is implied by the formula.
    pub fn fixed(&self, lit: i32) -> Result<i8, MyCadicalError> {
        let res = self.cadical.fixed(lit).map(|v| v as _)?;
        Ok(res)
    }

    /// Number of variables.
    pub fn vars(&self) -> i64 {
        self.cadical.vars()
    }

    /// Number of active variables.
    pub fn active(&self) -> i64 {
        self.cadical.active()
    }

    /// Number of active redundant clauses.
    pub fn redundant(&self) -> i64 {
        self.cadical.redundant()
    }

    /// Number of active irredundant clauses.
    pub fn irredundant(&self) -> i64 {
        self.cadical.irredundant()
    }

    /// Number of conflicts.
    pub fn conflicts(&self) -> i64 {
        self.cadical.conflicts()
    }

    /// Number of decisions.
    pub fn decisions(&self) -> i64 {
        self.cadical.decisions()
    }

    /// Number of restarts.
    pub fn restarts(&self) -> i64 {
        self.cadical.restarts()
    }

    /// Number of propagations.
    pub fn propagations(&self) -> i64 {
        self.cadical.propagations()
    }

    pub fn propcheck(&self, lits: Vec<i32>) -> (bool, u64) {
        self.cadical.propcheck(&lits, false, false, false)
    }

    pub fn propagate(&self, lits: Vec<i32>) -> (bool, Vec<i32>) {
        let (res, num_propagated) = self.cadical.propcheck(&lits, false, true, false);
        let propagated = self.cadical.propcheck_get_propagated();
        assert_eq!(num_propagated, propagated.len() as u64);
        (res, propagated)
    }
}

struct MyCadicalError(pub CadicalError);

impl From<CadicalError> for MyCadicalError {
    fn from(err: CadicalError) -> Self {
        Self(err)
    }
}

impl From<MyCadicalError> for PyErr {
    fn from(err: MyCadicalError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}
