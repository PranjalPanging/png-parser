pub mod chunk;
pub mod chunk_type;
pub mod commands;
pub mod crypto;
pub mod error;
pub mod format;
pub mod header;
pub mod pixel;
pub mod png;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use crate::commands::mode;

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (
    input_path,
    file_path,
    output_path,
    password=None,
    mode_str="chunk",
    expires_days=None,
    expires_hours=None,
    expires_minutes=None,
    expires_seconds=None
))]
fn hide(
    input_path:      String,
    file_path:       String,
    output_path:     String,
    password:        Option<String>,
    mode_str:        &str,
    expires_days:    Option<i64>,
    expires_hours:   Option<i64>,
    expires_minutes: Option<i64>,
    expires_seconds: Option<i64>,
) -> PyResult<()> {
    let expiry = if expires_days.is_some()
        || expires_hours.is_some()
        || expires_minutes.is_some()
        || expires_seconds.is_some()
    {
        Some((expires_days, expires_hours, expires_minutes, expires_seconds))
    } else {
        None
    };

    mode::hide(
        &input_path,
        &file_path,
        &output_path,
        password.as_deref(),
        mode_str,
        expiry,
    ).map_err(pyo3::PyErr::from)
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (input_path, output_path, password=None))]
fn reveal(
    input_path:  String,
    output_path: String,
    password:    Option<String>,
) -> PyResult<String> {
    mode::reveal(&input_path, &output_path, password.as_deref())
        .map_err(pyo3::PyErr::from)
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (input_path, password=None))]
fn info(
    input_path: String,
    password:   Option<String>,
) -> PyResult<String> {
    let result = mode::info(&input_path, password.as_deref())
        .map_err(pyo3::PyErr::from)?;
    Ok(result.to_string())
}

#[cfg(feature = "python")]
#[pyfunction]
fn verify(input_path: String, password: String) -> PyResult<bool> {
    mode::verify(&input_path, &password)
        .map_err(pyo3::PyErr::from)
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (input_path, output_path, password=None))]
fn delete(
    input_path:  String,
    output_path: String,
    password:    Option<String>,
) -> PyResult<()> {
    mode::delete(&input_path, &output_path, password.as_deref())
        .map_err(pyo3::PyErr::from)
}

#[cfg(feature = "python")]
#[pyfunction]
fn reencrypt(
    input_path:   String,
    output_path:  String,
    old_password: String,
    new_password: String,
) -> PyResult<()> {
    mode::reencrypt(&input_path, &output_path, &old_password, &new_password)
        .map_err(pyo3::PyErr::from)
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (input_path, mode_str="chunk"))]
fn capacity(input_path: String, mode_str: &str) -> PyResult<usize> {
    mode::capacity(&input_path, mode_str)
        .map_err(pyo3::PyErr::from)
}

#[cfg(feature = "python")]
#[pyfunction]
fn fingerprint(input_path: String) -> PyResult<String> {
    mode::fingerprint(&input_path)
        .map_err(pyo3::PyErr::from)
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (
    file_path,
    carriers,
    output_dir,
    password=None,
    expires_days=None,
    expires_hours=None,
    expires_minutes=None,
    expires_seconds=None
))]
fn split(
    file_path:       String,
    carriers:        Vec<String>,
    output_dir:      String,
    password:        Option<String>,
    expires_days:    Option<i64>,
    expires_hours:   Option<i64>,
    expires_minutes: Option<i64>,
    expires_seconds: Option<i64>,
) -> PyResult<Vec<String>> {
    let expiry = if expires_days.is_some()
        || expires_hours.is_some()
        || expires_minutes.is_some()
        || expires_seconds.is_some()
    {
        Some((expires_days, expires_hours, expires_minutes, expires_seconds))
    } else {
        None
    };

    let carrier_refs: Vec<&str> = carriers.iter().map(|s| s.as_str()).collect();

    mode::split(
        &file_path,
        &carrier_refs,
        &output_dir,
        password.as_deref(),
        expiry,
    ).map_err(pyo3::PyErr::from)
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (inputs, output_path, password=None))]
fn merge(
    inputs:      Vec<String>,
    output_path: String,
    password:    Option<String>,
) -> PyResult<String> {
    let input_refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();
    mode::merge(&input_refs, &output_path, password.as_deref())
        .map_err(pyo3::PyErr::from)
}

#[cfg(feature = "python")]
#[pymodule]
fn png_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hide,        m)?)?;
    m.add_function(wrap_pyfunction!(reveal,      m)?)?;
    m.add_function(wrap_pyfunction!(info,        m)?)?;
    m.add_function(wrap_pyfunction!(verify,      m)?)?;
    m.add_function(wrap_pyfunction!(delete,      m)?)?;
    m.add_function(wrap_pyfunction!(reencrypt,   m)?)?;
    m.add_function(wrap_pyfunction!(capacity,    m)?)?;
    m.add_function(wrap_pyfunction!(fingerprint, m)?)?;
    m.add_function(wrap_pyfunction!(split,       m)?)?;
    m.add_function(wrap_pyfunction!(merge,       m)?)?;
    m.add("__version__", "0.3.0")?;
    Ok(())
}