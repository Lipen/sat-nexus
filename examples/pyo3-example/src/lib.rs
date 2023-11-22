use pyo3::prelude::*;

pub fn backdoor_to_clauses(cubes: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
    // Note: `cubes` are "easy" tasks.
    Python::with_gil(|py| -> PyResult<_> {
        // let pyeda = PyModule::import(py, "pyeda")?;
        // println!("pyeda = {}", pyeda);
        // let pyeda_version: String = pyeda.getattr("__version__")?.extract()?;
        // println!("pyeda.__version__ = {:?}", pyeda_version);

        let py_common_code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/python_code/common.py"));
        let py_common = PyModule::from_code(py, py_common_code, "common.py", "common")?;
        let f: Py<PyAny> = py_common.getattr("backdoor_to_clauses")?.into();
        // println!("f = {}", f);

        let result = f.call1(py, (cubes,))?.extract(py)?;

        Ok(result)
    })
    .unwrap()
}
