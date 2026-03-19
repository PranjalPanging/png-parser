use image::{DynamicImage, RgbaImage};

use crate::error::{Error, Result};
const LENGTH_PREFIX_PIXELS: usize = 64;

const VARIANCE_THRESHOLD: f32 = 10.0;

fn bytes_to_bits(data: &[u8]) -> Vec<u8> {
    let mut bits = Vec::with_capacity(data.len() * 8);
    for byte in data {
        for shift in (0..8).rev() {
            bits.push((byte >> shift) & 1);
        }
    }
    bits
}

fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
    bits.chunks(8)
        .filter(|c| c.len() == 8)
        .map(|c| {
            c.iter()
                .enumerate()
                .fold(0u8, |acc, (i, &b)| acc | (b << (7 - i)))
        })
        .collect()
}

fn local_variance(img: &RgbaImage, cx: u32, cy: u32) -> f32 {
    let mut lums = [0f32; 9];
    let mut idx  = 0;

    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            let px = img.get_pixel(
                (cx as i32 + dx) as u32,
                (cy as i32 + dy) as u32,
            );
            lums[idx] = 0.299 * px[0] as f32
                      + 0.587 * px[1] as f32
                      + 0.114 * px[2] as f32;
            idx += 1;
        }
    }

    let mean = lums.iter().sum::<f32>() / 9.0;
    lums.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / 9.0
}

pub fn count_texture_pixels(img: &RgbaImage) -> usize {
    let (width, height) = img.dimensions();
    (1..height - 1)
        .flat_map(|y| (1..width - 1).map(move |x| (x, y)))
        .filter(|&(x, y)| local_variance(img, x, y) >= VARIANCE_THRESHOLD)
        .count()
}

pub fn pixel_capacity(image_path: &str) -> Result<usize> {
    let img  = load_rgba(image_path)?;
    let (w, h) = img.dimensions();

    if w < 3 || h < 3 {
        return Ok(0);
    }

    let texture_pixels = count_texture_pixels(&img);
    let usable         = texture_pixels.saturating_sub(LENGTH_PREFIX_PIXELS);
    Ok((usable * 3) / 8)
}

pub fn embed(
    input_path:  &str,
    output_path: &str,
    payload:     &[u8],
) -> Result<()> {
    let img  = load_image(input_path)?;
    let out  = embed_into_image(img, payload)?;
    out.save(output_path)
        .map_err(|e| Error::Io(
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        ))
}

pub fn embed_into_image(
    img:     DynamicImage,
    payload: &[u8],
) -> Result<DynamicImage> {
    let mut rgba          = img.to_rgba8();
    let (width, height)   = rgba.dimensions();

    if width < 3 || height < 3 {
        return Err(Error::InsufficientCapacity {
            needed:    payload.len(),
            available: 0,
        });
    }

    let texture_pixels = count_texture_pixels(&rgba);
    let available_bits = texture_pixels
        .saturating_sub(LENGTH_PREFIX_PIXELS) * 3;
    let needed_bits    = payload.len() * 8;

    if needed_bits > available_bits {
        return Err(Error::InsufficientCapacity {
            needed:    payload.len(),
            available: available_bits / 8,
        });
    }

    let len_bits  = bytes_to_bits(&(payload.len() as u64).to_be_bytes());
    let data_bits = bytes_to_bits(payload);

    let mut pixels = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)));
    for bit in &len_bits {
        let (x, y) = pixels
            .next()
            .ok_or(Error::InsufficientTexture)?;
        let mut px = rgba.get_pixel(x, y).0;
        px[0]      = (px[0] & 0xFE) | bit;
        rgba.put_pixel(x, y, image::Rgba(px));
    }

    let mut bit_idx = 0usize;

    'outer: for y in 1..height - 1 {
        for x in 1..width - 1 {
            if bit_idx >= data_bits.len() {
                break 'outer;
            }
            if local_variance(&rgba, x, y) < VARIANCE_THRESHOLD {
                continue;
            }
            let mut px = rgba.get_pixel(x, y).0;
            for channel in 0..3usize {
                if bit_idx < data_bits.len() {
                    px[channel] = (px[channel] & 0xFE) | data_bits[bit_idx];
                    bit_idx    += 1;
                }
            }
            rgba.put_pixel(x, y, image::Rgba(px));
        }
    }

    if bit_idx < data_bits.len() {
        return Err(Error::InsufficientTexture);
    }

    Ok(DynamicImage::ImageRgba8(rgba))
}
pub fn extract(input_path: &str) -> Result<Vec<u8>> {
    let img = load_image(input_path)?;
    extract_from_image(img)
}

pub fn extract_from_image(img: DynamicImage) -> Result<Vec<u8>> {
    let rgba          = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    if width < 3 || height < 3 {
        return Err(Error::NoPayload);
    }

    let mut pixels = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)));

    let mut len_bits = Vec::with_capacity(64);
    for _ in 0..LENGTH_PREFIX_PIXELS {
        let (x, y) = pixels
            .next()
            .ok_or(Error::NoPayload)?;
        let px = rgba.get_pixel(x, y).0;
        len_bits.push(px[0] & 1);
    }

    let len_bytes  = bits_to_bytes(&len_bits);
    let payload_len = u64::from_be_bytes(
        len_bytes
            .try_into()
            .map_err(|_| Error::CorruptHeader)?,
    ) as usize;

    if payload_len == 0 {
        return Err(Error::NoPayload);
    }
    let max_possible = (width * height * 3 / 8) as usize;
    if payload_len > max_possible {
        return Err(Error::CorruptHeader);
    }

    let total_bits = payload_len * 8;
    let mut data_bits = Vec::with_capacity(total_bits);

    'outer: for y in 1..height - 1 {
        for x in 1..width - 1 {
            if data_bits.len() >= total_bits {
                break 'outer;
            }
            if local_variance(&rgba, x, y) < VARIANCE_THRESHOLD {
                continue;
            }
            let px = rgba.get_pixel(x, y).0;
            for channel in 0..3usize {
                if data_bits.len() < total_bits {
                    data_bits.push(px[channel] & 1);
                }
            }
        }
    }

    if data_bits.len() < total_bits {
        return Err(Error::TruncatedChunk);
    }

    Ok(bits_to_bytes(&data_bits))
}
fn load_image(path: &str) -> Result<DynamicImage> {
    image::open(path).map_err(|e| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })
}
fn load_rgba(path: &str) -> Result<RgbaImage> {
    Ok(load_image(path)?.to_rgba8())
}