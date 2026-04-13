use std::path::PathBuf;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

fn to_py_err(e: anyhow::Error) -> PyErr {
    PyErr::new::<PyRuntimeError, _>(e.to_string())
}

#[pyfunction]
fn run_cli(argv: Vec<String>) -> PyResult<()> {
    draw::run_cli(argv.iter().map(String::as_str)).map_err(to_py_err)
}

#[pyfunction]
fn new_document(name: String) -> PyResult<String> {
    let doc = draw_core::Document::new(name);
    serde_json::to_string(&doc)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
}

#[pyfunction]
fn load_document(path: String) -> PyResult<String> {
    let doc = draw_core::storage::load(&PathBuf::from(path)).map_err(to_py_err)?;
    serde_json::to_string(&doc)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
}

#[pyfunction]
fn save_document(json: String, path: String) -> PyResult<()> {
    let doc: draw_core::Document = serde_json::from_str(&json)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
    draw_core::storage::save(&doc, &PathBuf::from(path)).map_err(to_py_err)
}

#[pyfunction]
fn export_svg(json: String) -> PyResult<String> {
    let doc: draw_core::Document = serde_json::from_str(&json)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
    Ok(draw_core::export_svg(&doc))
}

#[pyfunction]
#[pyo3(signature = (json, scale=2.0))]
fn export_png(json: String, scale: f32) -> PyResult<Vec<u8>> {
    let doc: draw_core::Document = serde_json::from_str(&json)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
    draw_core::export_png_with_scale(&doc, scale).map_err(to_py_err)
}

#[pymodule]
mod core {
    use super::*;

    #[pymodule_init]
    fn module_init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(run_cli, m)?)?;
        m.add_function(wrap_pyfunction!(new_document, m)?)?;
        m.add_function(wrap_pyfunction!(load_document, m)?)?;
        m.add_function(wrap_pyfunction!(save_document, m)?)?;
        m.add_function(wrap_pyfunction!(export_svg, m)?)?;
        m.add_function(wrap_pyfunction!(export_png, m)?)?;
        Ok(())
    }
}
