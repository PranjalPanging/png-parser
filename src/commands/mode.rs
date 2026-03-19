use std::fs;
use sha2::{Digest, Sha256};

use crate::crypto;
use crate::error::{Error, Result};
use crate::format::{self, ImageFormat};
use crate::header::{Expiry, EmbedMode, PayloadHeader, ShardInfo};
use crate::pixel;
use crate::png::Png;

pub fn hide(
    input_path:  &str,
    file_path:   &str,
    output_path: &str,
    password:    Option<&str>,
    mode_str:    &str,
    expiry:      Option<(Option<i64>, Option<i64>, Option<i64>, Option<i64>)>,
) -> Result<()> {
    format::assert_lossless_output(output_path)?;

    let carrier_bytes = format::read_image_bytes(input_path)?;
    let img_format    = format::detect_format(input_path, &carrier_bytes)?;

    let embed_mode = EmbedMode::from_str(mode_str)
        .ok_or_else(|| Error::UnsupportedFormat(
            format!("unknown mode '{}' — use 'chunk' or 'pixel'", mode_str)
        ))?;

    if embed_mode == EmbedMode::Pixel
        && matches!(img_format, ImageFormat::WebP)
    {
        return Err(Error::UnsupportedFormat(
            "pixel mode not supported for WebP — use chunk mode or PNG/BMP/TIFF".to_string(),
        ));
    }

    let file_bytes = fs::read(file_path).map_err(Error::Io)?;
    let filename   = std::path::Path::new(file_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let expiry_cfg = match expiry {
        Some((d, h, m, s)) => Expiry::from_parts(d, h, m, s),
        None               => Expiry::Never,
    };

    let header = PayloadHeader::new(
        filename,
        file_bytes.len() as u64,
        embed_mode.clone(),
        expiry_cfg,
    );

    let blob = header.pack(&file_bytes)?;

    let envelope = crypto::pack(&blob, password)?;

    match embed_mode {
        EmbedMode::Chunk => {
            let out_bytes = format::embed_chunk(
                &carrier_bytes,
                &envelope,
                &img_format,
            )?;
            format::write_image_bytes(output_path, &out_bytes)?;
        }
        EmbedMode::Pixel => {
            pixel::embed(input_path, output_path, &envelope)?;
        }
    }

    Ok(())
}

pub fn reveal(
    input_path:  &str,
    output_path: &str,
    password:    Option<&str>,
) -> Result<String> {
    let envelope = extract_envelope(input_path)?;

    let blob = crypto::unpack(&envelope, password)
        .map(|(_, b)| b)?;

    let (header, file_bytes) = PayloadHeader::unpack(&blob)?;

    if header.version > crate::header::HEADER_VERSION {
        return Err(Error::UnsupportedVersion {
            found:     header.version,
            supported: crate::header::HEADER_VERSION,
        });
    }

    if header.expiry.is_expired() {
        return Err(Error::Expired);
    }

    let out = std::path::Path::new(output_path);
    let final_path = if out.is_dir() {
        out.join(&header.filename)
    } else {
        out.to_path_buf()
    };

    fs::write(&final_path, file_bytes).map_err(Error::Io)?;

    Ok(final_path.to_string_lossy().to_string())
}

pub fn info(
    input_path: &str,
    password:   Option<&str>,
) -> Result<PayloadInfo> {
    let envelope = extract_envelope(input_path)?;
    let fp       = crypto::fingerprint(&envelope);

    let is_encrypted = envelope
        .first()
        .map(|&b| b == crypto::ENVELOPE_VERSION)
        .unwrap_or(false);

    if !is_encrypted {
        let blob             = crypto::unpack(&envelope, None).map(|(_, b)| b)?;
        let (header, _bytes) = PayloadHeader::unpack(&blob)?;

        return Ok(PayloadInfo {
            has_payload:  true,
            encrypted:    false,
            filename:     Some(header.filename),
            file_size:    Some(header.file_size),
            mode:         Some(header.mode.as_str().to_string()),
            expires_at:   Some(header.expiry.to_display()),
            version:      Some(header.version),
            fingerprint:  fp,
            shard:        header.shard.map(|s| ShardDisplay {
                index: s.index,
                total: s.total,
            }),
        });
    }

    match password {
        Some(pw) => {
            match crypto::unpack(&envelope, Some(pw)) {
                Ok((_, blob)) => {
                    match PayloadHeader::unpack(&blob) {
                        Ok((header, _)) => Ok(PayloadInfo {
                            has_payload:  true,
                            encrypted:    true,
                            filename:     Some(header.filename),
                            file_size:    Some(header.file_size),
                            mode:         Some(header.mode.as_str().to_string()),
                            expires_at:   Some(header.expiry.to_display()),
                            version:      Some(header.version),
                            fingerprint:  fp,
                            shard:        header.shard.map(|s| ShardDisplay {
                                index: s.index,
                                total: s.total,
                            }),
                        }),
                        Err(_) => Err(Error::CorruptHeader),
                    }
                }
                Err(_) => Err(Error::WrongPassword),
            }
        }
        None => Ok(PayloadInfo {
            has_payload:  true,
            encrypted:    true,
            filename:     None,
            file_size:    None,
            mode:         None,
            expires_at:   None,
            version:      None,
            fingerprint:  fp,
            shard:        None,
        }),
    }
}

pub fn verify(input_path: &str, password: &str) -> Result<bool> {
    let envelope = extract_envelope(input_path)?;
    Ok(crypto::verify_password(password, &envelope))
}

pub fn delete(
    input_path:  &str,
    output_path: &str,
    password:    Option<&str>,
) -> Result<()> {
    let image_bytes = format::read_image_bytes(input_path)?;
    let img_format  = format::detect_format(input_path, &image_bytes)?;

    let envelope = format::extract_chunk(&image_bytes, &img_format)?;

    let is_encrypted = envelope
        .first()
        .map(|&b| b == crypto::ENVELOPE_VERSION)
        .unwrap_or(false);

    if is_encrypted {
        match password {
            None => return Err(Error::DeletePasswordRequired),
            Some(pw) => {
                if !crypto::verify_password(pw, &envelope) {
                    return Err(Error::DeleteWrongPassword);
                }
            }
        }
    }

    let out_bytes = format::remove_chunk(&image_bytes, &img_format, true)?;
    format::write_image_bytes(output_path, &out_bytes)?;

    Ok(())
}

pub fn reencrypt(
    input_path:   &str,
    output_path:  &str,
    old_password: &str,
    new_password: &str,
) -> Result<()> {
    let image_bytes = format::read_image_bytes(input_path)?;
    let img_format  = format::detect_format(input_path, &image_bytes)?;

    let old_envelope = format::extract_chunk(&image_bytes, &img_format)?;

    let new_envelope = crypto::reencrypt(old_password, new_password, &old_envelope)?;

    let stripped  = format::remove_chunk(&image_bytes, &img_format, false)?;
    let out_bytes = format::embed_chunk(&stripped, &new_envelope, &img_format)?;

    format::write_image_bytes(output_path, &out_bytes)?;
    Ok(())
}

pub fn capacity(input_path: &str, mode_str: &str) -> Result<usize> {
    let embed_mode = EmbedMode::from_str(mode_str)
        .ok_or_else(|| Error::UnsupportedFormat(
            format!("unknown mode '{}' — use 'chunk' or 'pixel'", mode_str)
        ))?;

    match embed_mode {
        EmbedMode::Chunk => {
            let bytes      = format::read_image_bytes(input_path)?;
            let img_format = format::detect_format(input_path, &bytes)?;
            Ok(format::chunk_capacity(&img_format))
        }
        EmbedMode::Pixel => {
            pixel::pixel_capacity(input_path)
        }
    }
}

pub fn fingerprint(input_path: &str) -> Result<String> {
    let envelope = extract_envelope(input_path)?;
    Ok(crypto::fingerprint(&envelope))
}

pub fn inspect(input_path: &str) -> Result<()> {
    let bytes = format::read_image_bytes(input_path)?;
    let png   = Png::try_from(bytes.as_slice())?;

    println!("{}", png);

    for (chunk_type, length) in png.summary() {
        let marker = if chunk_type == "stEg" {
            " ← steg payload"
        } else {
            ""
        };
        println!("  {:4}  {:>10} bytes{}", chunk_type, length, marker);
    }

    Ok(())
}

pub fn strip(input_path: &str, output_path: &str) -> Result<()> {
    let bytes = format::read_image_bytes(input_path)?;
    let mut png = Png::try_from(bytes.as_slice())?;

    for chunk_type in &["tEXt", "zTXt", "iTXt", "eXIf"] {
        png.remove_chunk(chunk_type);
    }

    format::write_image_bytes(output_path, &png.as_bytes())?;
    Ok(())
}
pub fn split(
    file_path:  &str,
    carriers:   &[&str],
    output_dir: &str,
    password:   Option<&str>,
    expiry:     Option<(Option<i64>, Option<i64>, Option<i64>, Option<i64>)>,
) -> Result<Vec<String>> {
    if carriers.is_empty() {
        return Err(Error::ShardCountMismatch {
            expected: 1,
            found:    0,
        });
    }

    let file_bytes = fs::read(file_path).map_err(Error::Io)?;
    let filename   = std::path::Path::new(file_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let file_hash: [u8; 32] = Sha256::digest(&file_bytes).into();

    let total      = carriers.len();
    let shard_size = (file_bytes.len() + total - 1) / total;
    let shards: Vec<&[u8]> = file_bytes
        .chunks(shard_size)
        .collect();

    if shards.len() != total {
        return Err(Error::ShardCountMismatch {
            expected: total,
            found:    shards.len(),
        });
    }

    let expiry_cfg = match expiry {
        Some((d, h, m, s)) => Expiry::from_parts(d, h, m, s),
        None               => Expiry::Never,
    };

    let out_dir  = std::path::Path::new(output_dir);
    let mut outputs = Vec::with_capacity(total);

    for (i, (carrier, shard)) in carriers.iter().zip(shards.iter()).enumerate() {
        let carrier_bytes = format::read_image_bytes(carrier)?;
        let img_format    = format::detect_format(carrier, &carrier_bytes)?;
        format::assert_lossless_output(carrier)?;

        let shard_info = ShardInfo {
            index:     i as u32,
            total:     total as u32,
            file_hash,
        };

        let header = PayloadHeader::new_shard(
            filename.clone(),
            file_bytes.len() as u64,
            EmbedMode::Chunk,
            expiry_cfg.clone(),
            shard_info,
        );

        let blob     = header.pack(shard)?;
        let envelope = crypto::pack(&blob, password)?;

        let out_bytes = format::embed_chunk(
            &carrier_bytes,
            &envelope,
            &img_format,
        )?;

        let carrier_name = std::path::Path::new(carrier)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        let out_path = out_dir
            .join(format!("shard_{}_{}", i, carrier_name))
            .to_string_lossy()
            .to_string();

        format::write_image_bytes(&out_path, &out_bytes)?;
        outputs.push(out_path);
    }

    Ok(outputs)
}

pub fn merge(
    input_paths: &[&str],
    output_path: &str,
    password:    Option<&str>,
) -> Result<String> {
    if input_paths.is_empty() {
        return Err(Error::ShardCountMismatch {
            expected: 1,
            found:    0,
        });
    }

    let mut shards: Vec<(u32, u32, [u8; 32], Vec<u8>, String)> = Vec::new();

    for path in input_paths {
        let envelope         = extract_envelope(path)?;
        let blob             = crypto::unpack(&envelope, password).map(|(_, b)| b)?;
        let (header, bytes)  = PayloadHeader::unpack(&blob)?;

        if header.expiry.is_expired() {
            return Err(Error::Expired);
        }

        let shard_info = header.shard.ok_or(Error::CorruptHeader)?;

        shards.push((
            shard_info.index,
            shard_info.total,
            shard_info.file_hash,
            bytes.to_vec(),
            header.filename,
        ));
    }

    let expected_total = shards[0].1;
    let expected_hash  = shards[0].2;
    let filename       = shards[0].4.clone();

    for (_, total, hash, _, _) in &shards {
        if *total != expected_total {
            return Err(Error::ShardCountMismatch {
                expected: expected_total as usize,
                found:    *total as usize,
            });
        }
        if *hash != expected_hash {
            return Err(Error::ShardHashMismatch);
        }
    }

    if shards.len() != expected_total as usize {
        return Err(Error::ShardCountMismatch {
            expected: expected_total as usize,
            found:    shards.len(),
        });
    }

    shards.sort_by_key(|(index, _, _, _, _)| *index);

    let mut assembled = Vec::new();
    for (_, _, _, data, _) in &shards {
        assembled.extend_from_slice(data);
    }

    let actual_hash: [u8; 32] = Sha256::digest(&assembled).into();
    if actual_hash != expected_hash {
        return Err(Error::ShardHashMismatch);
    }

    let out = std::path::Path::new(output_path);
    let final_path = if out.is_dir() {
        out.join(&filename)
    } else {
        out.to_path_buf()
    };

    fs::write(&final_path, &assembled).map_err(Error::Io)?;

    Ok(final_path.to_string_lossy().to_string())
}

fn extract_envelope(input_path: &str) -> Result<Vec<u8>> {
    let image_bytes = format::read_image_bytes(input_path)?;
    let img_format  = format::detect_format(input_path, &image_bytes)?;

    match format::extract_chunk(&image_bytes, &img_format) {
        Ok(envelope) => return Ok(envelope),
        Err(Error::NoPayload) => {}
        Err(e) => return Err(e),
    }

    pixel::extract(input_path)
}
#[derive(Debug)]
pub struct PayloadInfo {
    pub has_payload:  bool,
    pub encrypted:    bool,
    pub filename:     Option<String>,
    pub file_size:    Option<u64>,
    pub mode:         Option<String>,
    pub expires_at:   Option<String>,
    pub version:      Option<u8>,
    pub fingerprint:  String,
    pub shard:        Option<ShardDisplay>,
}

#[derive(Debug)]
pub struct ShardDisplay {
    pub index: u32,
    pub total: u32,
}

impl std::fmt::Display for PayloadInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "has_payload : {}", self.has_payload)?;
        writeln!(f, "encrypted   : {}", self.encrypted)?;
        writeln!(f, "fingerprint : {}", self.fingerprint)?;
        if let Some(ref name) = self.filename {
            writeln!(f, "filename    : {}", name)?;
        }
        if let Some(size) = self.file_size {
            writeln!(f, "file_size   : {} bytes", size)?;
        }
        if let Some(ref mode) = self.mode {
            writeln!(f, "mode        : {}", mode)?;
        }
        if let Some(ref exp) = self.expires_at {
            writeln!(f, "expires_at  : {}", exp)?;
        }
        if let Some(ref s) = self.shard {
            writeln!(f, "shard       : {}/{}", s.index + 1, s.total)?;
        }
        Ok(())
    }
}
