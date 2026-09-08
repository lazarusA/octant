//! Raster image encoding (PNG, JPEG, WebP) and color profile chunk injection.

use super::ExportFormat;
use std::io::Cursor;

/// Crops a sub-rectangle from a raw RGBA8 image buffer.
pub fn crop_rgba_buffer(
    source: &[u8],
    src_w: u32,
    src_h: u32,
    crop_x: u32,
    crop_y: u32,
    crop_w: u32,
    crop_h: u32,
) -> (Vec<u8>, u32, u32) {
    let crop_x = crop_x.min(src_w.saturating_sub(1));
    let crop_y = crop_y.min(src_h.saturating_sub(1));
    let crop_w = crop_w.min(src_w.saturating_sub(crop_x)).max(1);
    let crop_h = crop_h.min(src_h.saturating_sub(crop_y)).max(1);

    let bytes_per_pixel = 4usize;
    let mut out = vec![0u8; (crop_w as usize) * (crop_h as usize) * bytes_per_pixel];

    for row in 0..crop_h {
        let src_row = crop_y + row;
        let src_start = ((src_row * src_w + crop_x) as usize) * bytes_per_pixel;
        let src_end = src_start + (crop_w as usize) * bytes_per_pixel;

        let dst_start = (row as usize) * (crop_w as usize) * bytes_per_pixel;
        let dst_end = dst_start + (crop_w as usize) * bytes_per_pixel;

        if src_end <= source.len() && dst_end <= out.len() {
            out[dst_start..dst_end].copy_from_slice(&source[src_start..src_end]);
        }
    }

    (out, crop_w, crop_h)
}

/// Encodes raw RGBA8 image pixels into PNG, JPEG, or WebP byte streams.
pub fn encode_raster_image(
    rgba: &[u8],
    width: u32,
    height: u32,
    format: ExportFormat,
    quality: u8,
) -> Result<Vec<u8>, String> {
    if rgba.len() != (width as usize) * (height as usize) * 4 {
        return Err(format!(
            "Invalid RGBA buffer length: expected {}, got {}",
            width * height * 4,
            rgba.len()
        ));
    }

    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);

    match format {
        ExportFormat::Png => {
            let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
                .ok_or_else(|| "Failed to construct RGBA image for PNG encoding".to_string())?;
            img.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)
                .map_err(|e| format!("PNG encoding error: {}", e))?;
            buffer = inject_display_p3_chunks(&buffer)?;
        }
        ExportFormat::Jpeg => {
            let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
                .ok_or_else(|| "Failed to construct RGBA image for JPEG encoding".to_string())?;
            let rgb = image::DynamicImage::ImageRgba8(img).to_rgb8();
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(cursor, quality.clamp(1, 100));
            encoder
                .encode_image(&rgb)
                .map_err(|e| format!("JPEG encoding error: {}", e))?;
        }
        ExportFormat::Webp => {
            let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
                .ok_or_else(|| "Failed to construct RGBA image for WebP encoding".to_string())?;
            img.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::WebP)
                .map_err(|e| format!("WebP encoding error: {}", e))?;
        }
        ExportFormat::Svg | ExportFormat::Pdf => {
            return Err("Vector formats (SVG/PDF) use dedicated generators".to_string());
        }
    }

    Ok(buffer)
}

/// Injects Apple Display P3 color primaries (`cHRM`) and Gamma 2.2 (`gAMA`) chunks into a raw PNG byte stream.
pub fn inject_display_p3_chunks(raw_png: &[u8]) -> Result<Vec<u8>, String> {
    const PNG_HEADER: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if raw_png.len() < 33 || &raw_png[0..8] != PNG_HEADER {
        return Err("Invalid PNG data header".to_string());
    }

    let ihdr_end = 33;
    if raw_png.len() < ihdr_end || &raw_png[12..16] != b"IHDR" {
        return Err("Corrupted or unexpected PNG IHDR chunk structure".to_string());
    }

    let mut chrm_data = Vec::with_capacity(32);
    chrm_data.extend_from_slice(&31270u32.to_be_bytes());
    chrm_data.extend_from_slice(&32900u32.to_be_bytes());
    chrm_data.extend_from_slice(&68000u32.to_be_bytes());
    chrm_data.extend_from_slice(&32000u32.to_be_bytes());
    chrm_data.extend_from_slice(&26500u32.to_be_bytes());
    chrm_data.extend_from_slice(&69000u32.to_be_bytes());
    chrm_data.extend_from_slice(&15000u32.to_be_bytes());
    chrm_data.extend_from_slice(&6000u32.to_be_bytes());
    let chrm_chunk = build_png_chunk(b"cHRM", &chrm_data);

    let mut gama_data = Vec::with_capacity(4);
    gama_data.extend_from_slice(&45455u32.to_be_bytes());
    let gama_chunk = build_png_chunk(b"gAMA", &gama_data);

    let mut output = Vec::with_capacity(raw_png.len() + chrm_chunk.len() + gama_chunk.len());
    output.extend_from_slice(&raw_png[..ihdr_end]);
    output.extend_from_slice(&chrm_chunk);
    output.extend_from_slice(&gama_chunk);
    output.extend_from_slice(&raw_png[ihdr_end..]);

    Ok(output)
}

fn png_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn build_png_chunk(chunk_type: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut chunk = Vec::with_capacity(4 + 4 + data.len() + 4);
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(chunk_type);
    chunk.extend_from_slice(data);

    let mut crc_buf = Vec::with_capacity(4 + data.len());
    crc_buf.extend_from_slice(chunk_type);
    crc_buf.extend_from_slice(data);
    let crc = png_crc32(&crc_buf);
    chunk.extend_from_slice(&crc.to_be_bytes());

    chunk
}
