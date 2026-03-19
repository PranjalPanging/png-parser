use pyo3::prelude::*;
use crate::commands::mode;

#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;

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
    let expiry = build_expiry(expires_days, expires_hours, expires_minutes, expires_seconds);
    mode::hide(&input_path, &file_path, &output_path, password.as_deref(), mode_str, expiry)
        .map_err(pyo3::PyErr::from)
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
fn info(input_path: String, password: Option<String>) -> PyResult<String> {
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
    let expiry       = build_expiry(expires_days, expires_hours, expires_minutes, expires_seconds);
    let carrier_refs: Vec<&str> = carriers.iter().map(|s| s.as_str()).collect();
    mode::split(&file_path, &carrier_refs, &output_dir, password.as_deref(), expiry)
        .map_err(pyo3::PyErr::from)
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

// ── WASM bindings ─────────────────────────────────────────────

#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn hide_js(
    contents:        Vec<u8>,
    file_bytes:      Vec<u8>,
    filename:        &str,
    password:        Option<String>,
    mode_str:        &str,
    expires_days:    Option<i64>,
    expires_hours:   Option<i64>,
    expires_minutes: Option<i64>,
    expires_seconds: Option<i64>,
) -> Result<Vec<u8>, JsValue> {
    use crate::header::{EmbedMode, Expiry, PayloadHeader};

    let embed_mode = EmbedMode::from_str(mode_str)
        .ok_or_else(|| JsValue::from_str("invalid mode — use chunk or pixel"))?;

    let expiry_cfg = match build_expiry(expires_days, expires_hours, expires_minutes, expires_seconds) {
        Some((d, h, m, s)) => Expiry::from_parts(d, h, m, s),
        None               => Expiry::Never,
    };

    let header = PayloadHeader::new(
        filename.to_string(),
        file_bytes.len() as u64,
        embed_mode.clone(),
        expiry_cfg,
    );

    let blob = header.pack(&file_bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let envelope = crate::crypto::pack(&blob, password.as_deref())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    match embed_mode {
        EmbedMode::Chunk => {
            let img_format = crate::format::detect_format("input.png", &contents)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            crate::format::embed_chunk(&contents, &envelope, &img_format)
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }
        EmbedMode::Pixel => {
            let img = image::load_from_memory(&contents)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let out = crate::pixel::embed_into_image(img, &envelope)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let mut buf = Vec::new();
            out.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            Ok(buf)
        }
    }
}

#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn reveal_js(
    contents: Vec<u8>,
    password: Option<String>,
) -> Result<js_sys::Uint8Array, JsValue> {
    let img_format = crate::format::detect_format("input.png", &contents)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let envelope = crate::format::extract_chunk(&contents, &img_format)
        .or_else(|_| {
            let img = image::load_from_memory(&contents)
                .map_err(|e| crate::error::Error::Io(
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                ))?;
            crate::pixel::extract_from_image(img)
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let (_, blob) = crate::crypto::unpack(&envelope, password.as_deref())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let (header, file_bytes) = crate::header::PayloadHeader::unpack(&blob)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    if header.expiry.is_expired() {
        return Err(JsValue::from_str("payload has expired"));
    }

    Ok(js_sys::Uint8Array::from(file_bytes))
}

#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn info_js(
    contents: Vec<u8>,
    password: Option<String>,
) -> Result<String, JsValue> {
    let img_format = crate::format::detect_format("input.png", &contents)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let envelope = crate::format::extract_chunk(&contents, &img_format)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let fp          = crate::crypto::fingerprint(&envelope);
    let is_encrypted = envelope.first().map(|&b| b == crate::crypto::ENVELOPE_VERSION).unwrap_or(false);

    if !is_encrypted {
        let (_, blob) = crate::crypto::unpack(&envelope, None)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let (header, _) = crate::header::PayloadHeader::unpack(&blob)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        return Ok(format!(
            r#"{{"encrypted":false,"filename":"{}","file_size":{},"expires_at":"{}","fingerprint":"{}"}}"#,
            header.filename,
            header.file_size,
            header.expiry.to_display(),
            fp,
        ));
    }

    match password {
        Some(pw) => {
            let (_, blob) = crate::crypto::unpack(&envelope, Some(&pw))
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let (header, _) = crate::header::PayloadHeader::unpack(&blob)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            Ok(format!(
                r#"{{"encrypted":true,"filename":"{}","file_size":{},"expires_at":"{}","fingerprint":"{}"}}"#,
                header.filename,
                header.file_size,
                header.expiry.to_display(),
                fp,
            ))
        }
        None => Ok(format!(
            r#"{{"encrypted":true,"filename":null,"file_size":null,"fingerprint":"{}"}}"#,
            fp,
        )),
    }
}

#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn verify_js(contents: Vec<u8>, password: &str) -> Result<bool, JsValue> {
    let img_format = crate::format::detect_format("input.png", &contents)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let envelope = crate::format::extract_chunk(&contents, &img_format)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(crate::crypto::verify_password(password, &envelope))
}

#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn delete_js(
    contents: Vec<u8>,
    password: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    let img_format = crate::format::detect_format("input.png", &contents)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let envelope = crate::format::extract_chunk(&contents, &img_format)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let is_encrypted = envelope.first().map(|&b| b == crate::crypto::ENVELOPE_VERSION).unwrap_or(false);

    if is_encrypted {
        match &password {
            None => return Err(JsValue::from_str("password required to delete encrypted payload")),
            Some(pw) => {
                if !crate::crypto::verify_password(pw, &envelope) {
                    return Err(JsValue::from_str("wrong password"));
                }
            }
        }
    }

    crate::format::remove_chunk(&contents, &img_format, true)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn reencrypt_js(
    contents:     Vec<u8>,
    old_password: &str,
    new_password: &str,
) -> Result<Vec<u8>, JsValue> {
    let img_format = crate::format::detect_format("input.png", &contents)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let old_envelope = crate::format::extract_chunk(&contents, &img_format)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let new_envelope = crate::crypto::reencrypt(old_password, new_password, &old_envelope)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let stripped = crate::format::remove_chunk(&contents, &img_format, false)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    crate::format::embed_chunk(&stripped, &new_envelope, &img_format)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn fingerprint_js(contents: Vec<u8>) -> Result<String, JsValue> {
    let img_format = crate::format::detect_format("input.png", &contents)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let envelope = crate::format::extract_chunk(&contents, &img_format)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(crate::crypto::fingerprint(&envelope))
}

#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn capacity_js(contents: Vec<u8>, mode_str: &str) -> Result<usize, JsValue> {
    use crate::header::EmbedMode;
    let embed_mode = EmbedMode::from_str(mode_str)
        .ok_or_else(|| JsValue::from_str("invalid mode — use chunk or pixel"))?;
    match embed_mode {
        EmbedMode::Chunk => {
            let img_format = crate::format::detect_format("input.png", &contents)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            Ok(crate::format::chunk_capacity(&img_format))
        }
        EmbedMode::Pixel => {
            let img = image::load_from_memory(&contents)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let rgba   = img.to_rgba8();
            let usable = crate::pixel::count_texture_pixels(&rgba)
                .saturating_sub(64);
            Ok((usable * 3) / 8)
        }
    }
}

fn build_expiry(
    d: Option<i64>,
    h: Option<i64>,
    m: Option<i64>,
    s: Option<i64>,
) -> Option<(Option<i64>, Option<i64>, Option<i64>, Option<i64>)> {
    if d.is_some() || h.is_some() || m.is_some() || s.is_some() {
        Some((d, h, m, s))
    } else {
        None
    }
}