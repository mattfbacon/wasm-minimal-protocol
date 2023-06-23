wasm_minimal_protocol::initiate_protocol!();
use pyo3::prelude::*;

#[wasm_minimal_protocol::wasm_func]
pub fn exec(code: &str) -> String {
    pyo3::prepare_freethreaded_python();
    let mut s = String::default();
    let x = Python::with_gil(|py| -> PyResult<()> {
        // let user: String = py.eval(code, None, Some(&locals))?.extract()?;
        s = py
            .eval(code, None, None)
            .unwrap()
            .str()
            .map(|x| x.to_string())
            .unwrap_or_else(|e| e.to_string());
        Ok(())
    });
    s
}
