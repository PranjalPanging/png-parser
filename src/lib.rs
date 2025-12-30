use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub mod chunk;
pub mod chunk_type;
pub mod commands;
pub mod png;

pub use chunk::Chunk;
pub use chunk_type::ChunkType;
pub use png::Png;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid chunk type")]
    InvalidChunkType,
    #[error("Invalid PNG signature")]
    InvalidSignature,
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "python")]
#[pyfunction]
fn hide(file_path: String, message: String) -> String {
    let chunk_name = b"stEg";

    let mut file = match OpenOptions::new().append(true).open(&file_path) {
        Ok(f) => f,
        Err(_) => return "Error: Could not open file.".to_string(),
    };

    let bytes = message.as_bytes();
    let length = bytes.len() as u32;

    if file.write_all(&length.to_be_bytes()).is_err()
        || file.write_all(chunk_name).is_err()
        || file.write_all(bytes).is_err()
        || file.write_all(&[0, 0, 0, 0]).is_err()
    {
        return "Error: Failed to write to image.".to_string();
    }

    "Success: Message hidden!".to_string()
}

#[cfg(feature = "python")]
#[pyfunction]
fn read(file_path: String) -> String {
    let target_chunk = "stEg";

    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(_) => return "Error: File not found.".to_string(),
    };

    let _ = file.seek(SeekFrom::Start(8));

    let mut buffer = [0u8; 4];
    loop {
        if file.read_exact(&mut buffer).is_err() {
            break;
        }
        let length = u32::from_be_bytes(buffer);

        let mut type_buf = [0u8; 4];
        if file.read_exact(&mut type_buf).is_err() {
            break;
        }
        let chunk_type = String::from_utf8_lossy(&type_buf);

        if chunk_type == target_chunk {
            let mut data = vec![0u8; length as usize];
            let _ = file.read_exact(&mut data);
            return String::from_utf8_lossy(&data).to_string();
        } else {
            let _ = file.seek(SeekFrom::Current(length as i64 + 4));
        }
    }

    "Error: No secret message found.".to_string()
}

#[cfg(feature = "python")]
#[pyfunction]
fn delete(file_path: String) -> String {
    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(_) => return "Error: Could not open file.".to_string(),
    };

    let mut contents = Vec::new();
    if file.read_to_end(&mut contents).is_err() {
        return "Error: Could not read file contents.".to_string();
    }

    let iend_signature = b"IEND";

    if let Some(pos) = contents
        .windows(4)
        .position(|window| window == iend_signature)
    {
        let end_of_png = pos + 8;

        let clean_png = &contents[..end_of_png];

        if std::fs::write(&file_path, clean_png).is_err() {
            return "Error: Could not save the clean file.".to_string();
        }
        "Success: Secret message deleted!".to_string()
    } else {
        "Error: Valid PNG structure not found (no IEND chunk).".to_string()
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn your_wasm_function() {
    #[cfg(feature = "js")]
    #[wasm_bindgen]
    pub fn hide_js(contents: Vec<u8>, message: &str) -> std::result::Result<Vec<u8>, JsError> {
        use crate::chunk::Chunk;
        use crate::chunk_type::ChunkType;
        use crate::png::Png;
        use std::str::FromStr;

        let mut png = Png::try_from(&contents[..])
            .map_err(|e| JsError::new(&format!("Failed to parse PNG: {}", e)))?;

        let chunk_type = ChunkType::from_str("stEg")
            .map_err(|e| JsError::new(&format!("Invalid chunk type: {}", e)))?;

        let chunk = Chunk::new(chunk_type, message.as_bytes().to_vec());
        png.append_chunk(chunk);

        Ok(png.as_bytes())
    }

    #[cfg(feature = "js")]
    #[wasm_bindgen]
    pub fn read_js(contents: Vec<u8>) -> std::result::Result<String, JsError> {
        use crate::png::Png;

        let png = Png::try_from(&contents[..])
            .map_err(|e| JsError::new(&format!("Failed to parse PNG: {}", e)))?;

        let target_chunk = png
            .chunks()
            .iter()
            .find(|c| c.chunk_type().to_string() == "stEg")
            .ok_or_else(|| JsError::new("No hidden message found (stEg chunk missing)"))?;

        let message = String::from_utf8(target_chunk.data().to_vec())
            .map_err(|_| JsError::new("Hidden data is not valid UTF-8"))?;

        Ok(message)
    }

    #[cfg(feature = "js")]
    #[wasm_bindgen]
    pub fn delete_js(contents: Vec<u8>) -> std::result::Result<Vec<u8>, JsError> {
        use crate::png::Png;

        let mut png = Png::try_from(&contents[..])
            .map_err(|e| JsError::new(&format!("Failed to parse PNG: {}", e)))?;

        png.remove_chunk("stEg");

        Ok(png.as_bytes())
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn png_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hide, m)?)?;
    m.add_function(wrap_pyfunction!(read, m)?)?;
    m.add_function(wrap_pyfunction!(delete, m)?)?;
    Ok(())
}
