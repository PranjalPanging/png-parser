use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use argon2::Argon2;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use rand::{thread_rng, RngCore};
use std::io::{Read, Write};

use crate::error::{Error, Result};

pub const ENVELOPE_VERSION:    u8    = 0x03;
pub const SALT_LEN:            usize = 32;
pub const NONCE_LEN:           usize = 12;
pub const CT_LEN_SIZE:         usize = 8;
pub const ENVELOPE_HEADER_LEN: usize = 1 + SALT_LEN + NONCE_LEN + CT_LEN_SIZE;

fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data)
        .map_err(|e| Error::CompressionFailed(e.to_string()))?;
    encoder.finish()
        .map_err(|e| Error::CompressionFailed(e.to_string()))
}

fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out     = Vec::new();
    decoder.read_to_end(&mut out)
        .map_err(|e| Error::DecompressionFailed(e.to_string()))?;
    Ok(out)
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| Error::KeyDerivation(e.to_string()))?;
    Ok(key)
}

pub fn encrypt(password: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
    let mut salt        = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    thread_rng().fill_bytes(&mut salt);
    thread_rng().fill_bytes(&mut nonce_bytes);

    let key_bytes  = derive_key(password, &salt)?;
    let cipher     = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let nonce      = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| Error::EncryptionFailed)?;

    let mut envelope = Vec::with_capacity(ENVELOPE_HEADER_LEN + ciphertext.len());
    envelope.push(ENVELOPE_VERSION);
    envelope.extend_from_slice(&salt);
    envelope.extend_from_slice(&nonce_bytes);
    envelope.extend_from_slice(&(ciphertext.len() as u64).to_be_bytes());
    envelope.extend_from_slice(&ciphertext);
    Ok(envelope)
}

pub fn decrypt(password: &str, envelope: &[u8]) -> Result<Vec<u8>> {
    if envelope.len() < ENVELOPE_HEADER_LEN {
        return Err(Error::CorruptHeader);
    }

    let version = envelope[0];
    if version != ENVELOPE_VERSION {
        return Err(Error::UnsupportedVersion {
            found:     version,
            supported: ENVELOPE_VERSION,
        });
    }

    let salt        = &envelope[1..1 + SALT_LEN];
    let nonce_off   = 1 + SALT_LEN;
    let nonce_bytes = &envelope[nonce_off..nonce_off + NONCE_LEN];
    let ct_len_off  = nonce_off + NONCE_LEN;
    let ct_len      = u64::from_be_bytes(
        envelope[ct_len_off..ct_len_off + CT_LEN_SIZE]
            .try_into()
            .map_err(|_| Error::CorruptHeader)?,
    ) as usize;
    let ct_start   = ct_len_off + CT_LEN_SIZE;

    if envelope.len() < ct_start + ct_len {
        return Err(Error::TruncatedChunk);
    }

    let ciphertext = &envelope[ct_start..ct_start + ct_len];
    let key_bytes  = derive_key(password, salt)?;
    let cipher     = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let nonce      = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| Error::WrongPassword)
}

pub fn verify_password(password: &str, envelope: &[u8]) -> bool {
    decrypt(password, envelope).is_ok()
}

pub fn compress_and_encrypt(password: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
    let compressed = compress(plaintext)?;
    encrypt(password, &compressed)
}

pub fn decrypt_and_decompress(password: &str, envelope: &[u8]) -> Result<Vec<u8>> {
    let compressed = decrypt(password, envelope)?;
    decompress(&compressed)
}

pub fn reencrypt(
    old_password: &str,
    new_password: &str,
    envelope:     &[u8],
) -> Result<Vec<u8>> {
    let plaintext = decrypt(old_password, envelope)?;
    encrypt(new_password, &plaintext)
}

pub fn fingerprint(envelope: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(envelope);
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

pub fn compress_only(plaintext: &[u8]) -> Result<Vec<u8>> {
    let mut out        = vec![0x00u8];
    let compressed     = compress(plaintext)?;
    out.extend_from_slice(&compressed);
    Ok(out)
}

pub fn decompress_only(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() || data[0] != 0x00 {
        return Err(Error::CorruptHeader);
    }
    decompress(&data[1..])
}

pub fn unpack(
    raw:      &[u8],
    password: Option<&str>,
) -> Result<(bool, Vec<u8>)> {
    if raw.is_empty() {
        return Err(Error::CorruptHeader);
    }
    match raw[0] {
        0x00 => {
            let plaintext = decompress_only(raw)?;
            Ok((false, plaintext))
        }
        ENVELOPE_VERSION => {
            let pw        = password.ok_or(Error::PasswordRequired)?;
            let plaintext = decrypt_and_decompress(pw, raw)?;
            Ok((true, plaintext))
        }
        v => Err(Error::UnsupportedVersion {
            found:     v,
            supported: ENVELOPE_VERSION,
        }),
    }
}

pub fn pack(plaintext: &[u8], password: Option<&str>) -> Result<Vec<u8>> {
    match password {
        Some(pw) => compress_and_encrypt(pw, plaintext),
        None     => compress_only(plaintext),
    }
}