use std::path::Path;

use crate::chunk::Chunk;
use crate::chunk_type::ChunkType;
use crate::error::{Error, Result};
use crate::png::Png;

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Png,
    Bmp,
    Tiff,
    WebP,
}

impl ImageFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageFormat::Png  => "PNG",
            ImageFormat::Bmp  => "BMP",
            ImageFormat::Tiff => "TIFF",
            ImageFormat::WebP => "WebP",
        }
    }
}

const PNG_MAGIC:  [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const BMP_MAGIC:  [u8; 2] = [0x42, 0x4D];  
const TIFF_LE:    [u8; 4] = [0x49, 0x49, 0x2A, 0x00];
const TIFF_BE:    [u8; 4] = [0x4D, 0x4D, 0x00, 0x2A];
const WEBP_MAGIC: [u8; 4] = [0x52, 0x49, 0x46, 0x46];
const JPEG_MAGIC: [u8; 3] = [0xFF, 0xD8, 0xFF];
pub fn detect_format(path: &str, bytes: &[u8]) -> Result<ImageFormat> {
    if bytes.starts_with(&PNG_MAGIC) {
        return Ok(ImageFormat::Png);
    }
    if bytes.starts_with(&BMP_MAGIC) {
        return Ok(ImageFormat::Bmp);
    }
    if bytes.starts_with(&TIFF_LE) || bytes.starts_with(&TIFF_BE) {
        return Ok(ImageFormat::Tiff);
    }
    if bytes.starts_with(&WEBP_MAGIC)
        && bytes.len() >= 12
        && &bytes[8..12] == b"WEBP"
    {
        return Ok(ImageFormat::WebP);
    }
    if bytes.starts_with(&JPEG_MAGIC) {
        return Err(Error::LossyFormat);
    }

    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "png"          => Ok(ImageFormat::Png),
        "bmp"          => Ok(ImageFormat::Bmp),
        "tiff" | "tif" => Ok(ImageFormat::Tiff),
        "webp"         => Ok(ImageFormat::WebP),
        "jpg" | "jpeg" => Err(Error::LossyFormat),
        other          => Err(Error::UnsupportedFormat(other.to_string())),
    }
}

pub fn assert_lossless_output(path: &str) -> Result<()> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "png" | "bmp" | "tiff" | "tif" | "webp" => Ok(()),
        "jpg" | "jpeg" => Err(Error::LossyFormat),
        other => Err(Error::UnsupportedFormat(other.to_string())),
    }
}

pub fn embed_chunk(
    image_bytes: &[u8],
    payload:     &[u8],
    format:      &ImageFormat,
) -> Result<Vec<u8>> {
    match format {
        ImageFormat::Png  => embed_png_chunk(image_bytes, payload),
        ImageFormat::Bmp  => embed_bmp_append(image_bytes, payload),
        ImageFormat::Tiff => embed_tiff_tag(image_bytes, payload),
        ImageFormat::WebP => embed_webp_chunk(image_bytes, payload),
    }
}

pub fn extract_chunk(
    image_bytes: &[u8],
    format:      &ImageFormat,
) -> Result<Vec<u8>> {
    match format {
        ImageFormat::Png  => extract_png_chunk(image_bytes),
        ImageFormat::Bmp  => extract_bmp_append(image_bytes),
        ImageFormat::Tiff => extract_tiff_tag(image_bytes),
        ImageFormat::WebP => extract_webp_chunk(image_bytes),
    }
}
pub fn remove_chunk(
    image_bytes: &[u8],
    format:      &ImageFormat,
    secure:      bool,
) -> Result<Vec<u8>> {
    match format {
        ImageFormat::Png  => remove_png_chunk(image_bytes, secure),
        ImageFormat::Bmp  => remove_bmp_append(image_bytes),
        ImageFormat::Tiff => remove_tiff_tag(image_bytes),
        ImageFormat::WebP => remove_webp_chunk(image_bytes),
    }
}

fn embed_png_chunk(bytes: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    let mut png = Png::try_from(bytes)?;
    if png.has_payload() {
        png.remove_chunk_secure("stEg");
    }

    let chunk = Chunk::new(ChunkType::steg(), payload.to_vec());
    png.insert_before_iend(chunk);
    Ok(png.as_bytes())
}

fn extract_png_chunk(bytes: &[u8]) -> Result<Vec<u8>> {
    let png = Png::try_from(bytes)?;
    let chunk = png
        .find_chunk("stEg")
        .ok_or(Error::NoPayload)?;
    Ok(chunk.data().to_vec())
}

fn remove_png_chunk(bytes: &[u8], secure: bool) -> Result<Vec<u8>> {
    let mut png = Png::try_from(bytes)?;
    if !png.has_payload() {
        return Err(Error::NoPayload);
    }
    if secure {
        png.remove_chunk_secure("stEg");
    } else {
        png.remove_chunk("stEg");
    }
    Ok(png.as_bytes())
}


const BMP_STEG_MAGIC: &[u8; 8] = b"stEgBMP\x00";

fn embed_bmp_append(bytes: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    let clean = strip_bmp_trailer(bytes);

    let mut out = clean.to_vec();
    out.extend_from_slice(BMP_STEG_MAGIC);
    out.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

fn extract_bmp_append(bytes: &[u8]) -> Result<Vec<u8>> {
    let pos = find_bmp_trailer(bytes).ok_or(Error::NoPayload)?;
    let after_magic = pos + BMP_STEG_MAGIC.len();

    if bytes.len() < after_magic + 8 {
        return Err(Error::TruncatedChunk);
    }

    let len = u64::from_be_bytes(
        bytes[after_magic..after_magic + 8]
            .try_into()
            .map_err(|_| Error::CorruptHeader)?,
    ) as usize;

    let data_start = after_magic + 8;
    if bytes.len() < data_start + len {
        return Err(Error::TruncatedChunk);
    }

    Ok(bytes[data_start..data_start + len].to_vec())
}

fn remove_bmp_append(bytes: &[u8]) -> Result<Vec<u8>> {
    if find_bmp_trailer(bytes).is_none() {
        return Err(Error::NoPayload);
    }
    Ok(strip_bmp_trailer(bytes).to_vec())
}

fn find_bmp_trailer(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(BMP_STEG_MAGIC.len())
        .rposition(|w| w == BMP_STEG_MAGIC)
}

fn strip_bmp_trailer(bytes: &[u8]) -> &[u8] {
    match find_bmp_trailer(bytes) {
        Some(pos) => &bytes[..pos],
        None      => bytes,
    }
}

const _TIFF_STEG_TAG: u16 = 0xEEEE;

fn embed_tiff_tag(bytes: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    const TIFF_STEG_MAGIC: &[u8; 8] = b"stEgTIF\x00";

    let clean = strip_trailer(bytes, TIFF_STEG_MAGIC);
    let mut out = clean.to_vec();
    out.extend_from_slice(TIFF_STEG_MAGIC);
    out.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

fn extract_tiff_tag(bytes: &[u8]) -> Result<Vec<u8>> {
    const TIFF_STEG_MAGIC: &[u8; 8] = b"stEgTIF\x00";
    extract_trailer(bytes, TIFF_STEG_MAGIC)
}

fn remove_tiff_tag(bytes: &[u8]) -> Result<Vec<u8>> {
    const TIFF_STEG_MAGIC: &[u8; 8] = b"stEgTIF\x00";
    if find_trailer(bytes, TIFF_STEG_MAGIC).is_none() {
        return Err(Error::NoPayload);
    }
    Ok(strip_trailer(bytes, TIFF_STEG_MAGIC).to_vec())
}

const WEBP_STEG_FOURCC: &[u8; 4] = b"stEg";

fn embed_webp_chunk(bytes: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    let clean = remove_webp_steg(bytes);

    let mut steg_chunk = WEBP_STEG_FOURCC.to_vec();
    steg_chunk.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    steg_chunk.extend_from_slice(payload);
    if payload.len() % 2 != 0 {
        steg_chunk.push(0x00);
    }

    let mut out = clean;
    let new_riff_size = (out.len() - 8 + steg_chunk.len()) as u32;
    out[4..8].copy_from_slice(&new_riff_size.to_le_bytes());

    out.extend_from_slice(&steg_chunk);
    Ok(out)
}

fn extract_webp_chunk(bytes: &[u8]) -> Result<Vec<u8>> {
    find_webp_steg(bytes).ok_or(Error::NoPayload)
}

fn remove_webp_chunk(bytes: &[u8]) -> Result<Vec<u8>> {
    if find_webp_steg(bytes).is_none() {
        return Err(Error::NoPayload);
    }
    let out = remove_webp_steg(bytes);
    Ok(out)
}

fn find_webp_steg(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() < 12 {
        return None;
    }
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let fourcc = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes(
            bytes[pos + 4..pos + 8].try_into().ok()?
        ) as usize;
        if fourcc == WEBP_STEG_FOURCC {
            let data_start = pos + 8;
            let data_end   = data_start + size;
            if data_end <= bytes.len() {
                return Some(bytes[data_start..data_end].to_vec());
            }
        }
        pos += 8 + size + (size % 2);
    }
    None
}

fn remove_webp_steg(bytes: &[u8]) -> Vec<u8> {
    if bytes.len() < 12 {
        return bytes.to_vec();
    }
    let mut out  = bytes[..12].to_vec();
    let mut pos  = 12;
    let mut removed_size = 0usize;

    while pos + 8 <= bytes.len() {
        let fourcc = &bytes[pos..pos + 4];
        let size   = match bytes[pos + 4..pos + 8].try_into() {
            Ok(b) => u32::from_le_bytes(b) as usize,
            Err(_) => break,
        };
        let padded = size + (size % 2);
        let chunk_total = 8 + padded;

        if fourcc == WEBP_STEG_FOURCC {
            removed_size = chunk_total;
        } else {
            let end = (pos + chunk_total).min(bytes.len());
            out.extend_from_slice(&bytes[pos..end]);
        }
        pos += chunk_total;
    }

    if removed_size > 0 && out.len() >= 8 {
        let new_size = (out.len() as u32).saturating_sub(8);
        out[4..8].copy_from_slice(&new_size.to_le_bytes());
    }

    out
}


fn find_trailer<'a>(bytes: &'a [u8], magic: &[u8]) -> Option<usize> {
    bytes
        .windows(magic.len())
        .rposition(|w| w == magic)
}

fn strip_trailer<'a>(bytes: &'a [u8], magic: &[u8]) -> &'a [u8] {
    match find_trailer(bytes, magic) {
        Some(pos) => &bytes[..pos],
        None      => bytes,
    }
}

fn extract_trailer(bytes: &[u8], magic: &[u8]) -> Result<Vec<u8>> {
    let pos        = find_trailer(bytes, magic).ok_or(Error::NoPayload)?;
    let after      = pos + magic.len();

    if bytes.len() < after + 8 {
        return Err(Error::TruncatedChunk);
    }

    let len = u64::from_be_bytes(
        bytes[after..after + 8]
            .try_into()
            .map_err(|_| Error::CorruptHeader)?,
    ) as usize;

    let data_start = after + 8;
    if bytes.len() < data_start + len {
        return Err(Error::TruncatedChunk);
    }

    Ok(bytes[data_start..data_start + len].to_vec())
}

pub fn chunk_capacity(_format: &ImageFormat) -> usize {
    256 * 1024 * 1024
}

pub fn read_image_bytes(path: &str) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(Error::Io)
}
pub fn write_image_bytes(path: &str, bytes: &[u8]) -> Result<()> {
    std::fs::write(path, bytes).map_err(Error::Io)
}