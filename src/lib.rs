use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::Aead};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use rand::{RngCore, thread_rng};

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

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
#[pyo3(signature = (file_path, message, password=None))]
fn hide(file_path: String, message: String, password: Option<String>) -> String {
    let chunk_name = b"stEg";
    let mut payload = Vec::new();

    if let Some(pw) = password.filter(|p| !p.is_empty()) {
        payload.push(0x01);
        
        let mut salt = [0u8; 16];
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill_bytes(&mut salt);
        thread_rng().fill_bytes(&mut nonce_bytes);

        let mut key_bytes = [0u8; 32];
        pbkdf2_hmac::<Sha256>(pw.as_bytes(), &salt, 100_000, &mut key_bytes);
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = match cipher.encrypt(nonce, message.as_bytes()) {
            Ok(ct) => ct,
            Err(_) => return "Error: Encryption failed.".to_string(),
        };

        payload.extend_from_slice(&salt);
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);
    } else {
        payload.push(0x00);
        payload.extend_from_slice(message.as_bytes());
    }

    let mut file = match OpenOptions::new().append(true).open(&file_path) {
        Ok(f) => f,
        Err(_) => return "Error: Could not open file.".to_string(),
    };

    let length = payload.len() as u32;
    if file.write_all(&length.to_be_bytes()).is_err()
        || file.write_all(chunk_name).is_err()
        || file.write_all(&payload).is_err()
        || file.write_all(&[0, 0, 0, 0]).is_err() 
    {
        return "Error: Failed to write to image.".to_string();
    }

    "Success: Message hidden!".to_string()
}



#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (file_path, password=None))]
fn read(file_path: String, password: Option<String>) -> String {
    let target_chunk = "stEg";
    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(_) => return "Error: File not found.".to_string(),
    };

    let _ = file.seek(SeekFrom::Start(8));
    let mut buffer = [0u8; 4];

    loop {
        if file.read_exact(&mut buffer).is_err() { break; }
        let length = u32::from_be_bytes(buffer);
        let mut type_buf = [0u8; 4];
        if file.read_exact(&mut type_buf).is_err() { break; }
        
        if String::from_utf8_lossy(&type_buf) == target_chunk {
            let mut payload = vec![0u8; length as usize];
            let _ = file.read_exact(&mut payload);

            let flag = payload[0];
            let data = &payload[1..];

            if flag == 0x00 {
                return String::from_utf8_lossy(data).to_string();
            } else if flag == 0x01 {
                let pw = match password {
                    Some(p) if !p.is_empty() => p,
                    _ => return "Error: This message is encrypted. Password required.".to_string(),
                };

                let salt = &data[0..16];
                let nonce = Nonce::from_slice(&data[16..28]);
                let ciphertext = &data[28..];

                let mut key_bytes = [0u8; 32];
                pbkdf2_hmac::<Sha256>(pw.as_bytes(), salt, 100_000, &mut key_bytes);
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));

                return match cipher.decrypt(nonce, ciphertext) {
                    Ok(pt) => String::from_utf8_lossy(&pt).to_string(),
                    Err(_) => "Error: Incorrect password.".to_string(),
                };
            }
        } else {
            let _ = file.seek(SeekFrom::Current(length as i64 + 4));
        }
    }
    "Error: No message found.".to_string()
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

#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;


#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn hide_js(contents: Vec<u8>, message: &str, password: Option<String>) -> std::result::Result<Vec<u8>, JsValue> {
    use crate::png::Png;
    use crate::chunk::Chunk;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    let mut payload = Vec::new();

    if let Some(pw) = password.filter(|p| !p.is_empty()) {
        payload.push(0x01);
        let mut salt = [0u8; 16];
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill_bytes(&mut salt);
        thread_rng().fill_bytes(&mut nonce_bytes);

        let mut key_bytes = [0u8; 32];
        pbkdf2_hmac::<Sha256>(pw.as_bytes(), &salt, 100_000, &mut key_bytes);
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, message.as_bytes())
            .map_err(|_| JsValue::from_str("Encryption failed"))?;

        payload.extend_from_slice(&salt);
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);
    } else {
        payload.push(0x00);
        payload.extend_from_slice(message.as_bytes());
    }

    let mut png = Png::try_from(&contents[..])
        .map_err(|e| JsValue::from_str(&format!("Invalid PNG: {}", e)))?;
    
    let chunk = Chunk::new(
        ChunkType::from_str("stEg").unwrap(),
        payload
    );
    
    png.append_chunk(chunk);
    Ok(png.as_bytes())
}


#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn read_js(contents: Vec<u8>, password: Option<String>) -> std::result::Result<String, JsValue> {
    use crate::png::Png;

    let png = Png::try_from(&contents[..])
        .map_err(|e| JsValue::from_str(&format!("Invalid PNG: {}", e)))?;
    
    if let Some(chunk) = png.chunks().iter().find(|c| c.chunk_type().to_string() == "stEg") {
        let payload = chunk.data();
        if payload.is_empty() { return Err(JsValue::from_str("Empty chunk")); }

        let flag = payload[0];
        let data = &payload[1..];

        if flag == 0x00 {
            return Ok(String::from_utf8_lossy(data).to_string());
        } else {
            let pw = password.filter(|p| !p.is_empty())
                .ok_or_else(|| JsValue::from_str("Password required for encrypted message"))?;

            let salt = &data[0..16];
            let nonce_bytes = &data[16..28];
            let ciphertext = &data[28..];

            let mut key_bytes = [0u8; 32];
            pbkdf2_hmac::<Sha256>(pw.as_bytes(), salt, 100_000, &mut key_bytes);
            
            let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(nonce_bytes);

            let plaintext = cipher.decrypt(nonce, ciphertext)
                .map_err(|_| JsValue::from_str("Incorrect password or corrupted data"))?;
            
            return Ok(String::from_utf8_lossy(&plaintext).to_string());
        }
    }
    Err(JsValue::from_str("No secret message found"))
}


#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn delete_js(contents: Vec<u8>) -> std::result::Result<Vec<u8>, JsError> {
    use crate::png::Png;
    let mut png = Png::try_from(&contents[..])
        .map_err(|e| JsError::new(&format!("Failed to parse PNG: {}", e)))?;

    let has_chunk = png.chunks().iter().any(|c| c.chunk_type().to_string() == "stEg");

    if !has_chunk {
        return Err(JsError::new("No hidden message found in this image"));
    }

    png.remove_chunk("stEg");

    Ok(png.as_bytes())
}


#[cfg(feature = "python")]
#[pymodule]
fn png_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hide, m)?)?;
    m.add_function(wrap_pyfunction!(read, m)?)?;
    m.add_function(wrap_pyfunction!(delete, m)?)?;
    Ok(())
}
