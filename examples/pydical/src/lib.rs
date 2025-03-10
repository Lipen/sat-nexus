use std::fmt::{Display, Formatter};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use cadical::statik::{Cadical, CadicalError};

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

    pub fn assume(&self, lit: i32) -> Result<(), MyCadicalError> {
        self.cadical.assume(lit)?;
        Ok(())
    }

    /// Returns 10 for SAT.
    /// Returns 20 for UNSAT.
    /// Returns 0 for UNKNOWN.
    #[pyo3(signature = (assumptions=vec![]), text_signature = "(assumptions=[])")]
    pub fn solve(&mut self, assumptions: Vec<i32>) -> Result<i32, MyCadicalError> {
        for lit in assumptions {
            self.cadical.assume(lit)?;
        }
        let res = self.cadical.solve()?;
        Ok(res as i32)
    }

    pub fn val(&self, lit: i32) -> Result<bool, MyCadicalError> {
        let res = self.cadical.val(lit).map(|v| v.into())?;
        Ok(res)
    }

    pub fn failed(&self, lit: i32) -> Result<bool, MyCadicalError> {
        let res = self.cadical.failed(lit).map(|v| v.into())?;
        Ok(res)
    }

    pub fn fixed(&self, lit: i32) -> Result<i8, MyCadicalError> {
        let res = self.cadical.fixed(lit).map(|v| v as _)?;
        Ok(res)
    }

    pub fn propcheck(&self, lits: Vec<i32>) -> Result<(bool, Vec<i32>), MyCadicalError> {
        let (res, num_propagated) = self.cadical.propcheck(&lits, false, true, false);
        let propagated = self.cadical.propcheck_get_propagated();
        assert_eq!(num_propagated, propagated.len() as u64);
        Ok((res, propagated))
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
