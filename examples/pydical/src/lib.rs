use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use cadical::statik::{Cadical, CadicalError};

#[pymodule]
fn pydical(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pydical>()?;
    Ok(())
}

#[pyclass(unsendable)]
struct Pydical {
    cadical: Cadical,
}

#[pymethods]
impl Pydical {
    #[new]
    pub fn new() -> PyResult<Self> {
        Ok(Self { cadical: Cadical::new() })
    }

    pub fn signature(&self) -> PyResult<&str> {
        Ok(self.cadical.signature())
    }

    pub fn release(&mut self) -> PyResult<()> {
        Ok(self.cadical.release())
    }

    pub fn limit(&mut self, name: &str, limit: i32) -> PyResult<()> {
        Ok(self.cadical.limit(name, limit))
    }

    pub fn add(&mut self, lit: i32) -> PyResult<()> {
        Ok(self.cadical.add(lit))
    }

    pub fn assume(&mut self, lit: i32) -> Result<(), MyCadicalError> {
        Ok(self.cadical.assume(lit)?)
    }

    pub fn solve(&mut self) -> Result<i32, MyCadicalError> {
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

    pub fn propcheck(&mut self, lits: Vec<i32>) -> Result<(bool, Vec<i32>), MyCadicalError> {
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
