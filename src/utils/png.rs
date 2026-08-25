use image::RgbaImage;
use std::io::Cursor;
use std::path::Path;

/// Computes standard IEEE 802.3 CRC32 used in PNG chunks.
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

/// Builds a self-contained PNG chunk: `[length (4B)][type (4B)][data][crc32 (4B)]`.
fn build_png_chunk(chunk_type: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut chunk = Vec::with_capacity(4 + 4 + data.len() + 4);
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(chunk_type);
    chunk.extend_from_slice(data);

    // CRC is computed over the chunk type and chunk data (not including the length bytes)
    let mut crc_buf = Vec::with_capacity(4 + data.len());
    crc_buf.extend_from_slice(chunk_type);
    crc_buf.extend_from_slice(data);
    let crc = png_crc32(&crc_buf);
    chunk.extend_from_slice(&crc.to_be_bytes());

    chunk
}

/// Injects Apple Display P3 color primaries (`cHRM`) and Gamma 2.2 (`gAMA`) chunks into a raw PNG byte stream.
///
/// This ensures macOS Preview, Finder QuickLook, Safari, and Adobe Photoshop render the exact
/// wide-gamut saturation and contrast seen on Apple Retina displays.
pub fn inject_display_p3_chunks(raw_png: &[u8]) -> Result<Vec<u8>, String> {
    const PNG_HEADER: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if raw_png.len() < 33 || &raw_png[0..8] != PNG_HEADER {
        return Err("Invalid PNG data header".to_string());
    }

    // Standard IHDR chunk is at offset 8:
    // Length: 4 bytes (13) + Type: 4 bytes (IHDR) + Data: 13 bytes + CRC: 4 bytes = 25 bytes
    // Total offset after IHDR: 8 + 25 = 33 bytes.
    let ihdr_end = 33;
    if raw_png.len() < ihdr_end || &raw_png[12..16] != b"IHDR" {
        return Err("Corrupted or unexpected PNG IHDR chunk structure".to_string());
    }

    // 1. Display P3 Primary Chromaticities (`cHRM` chunk, D65 White Point + DCI-P3 Primaries)
    // Scaled by 100,000 as required by the PNG specification:
    // White point: x=0.3127, y=0.3290 -> 31270, 32900
    // Red:         x=0.6800, y=0.3200 -> 68000, 32000
    // Green:       x=0.2650, y=0.6900 -> 26500, 69000
    // Blue:        x=0.1500, y=0.0600 -> 15000,  6000
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

    // 2. Display Gamma (`gAMA` chunk, 1 / 2.2 = 0.45455 -> 45455)
    let mut gama_data = Vec::with_capacity(4);
    gama_data.extend_from_slice(&45455u32.to_be_bytes());
    let gama_chunk = build_png_chunk(b"gAMA", &gama_data);

    // Splice in cHRM and gAMA chunks immediately after IHDR
    let mut output = Vec::with_capacity(raw_png.len() + chrm_chunk.len() + gama_chunk.len());
    output.extend_from_slice(&raw_png[..ihdr_end]);
    output.extend_from_slice(&chrm_chunk);
    output.extend_from_slice(&gama_chunk);
    output.extend_from_slice(&raw_png[ihdr_end..]);

    Ok(output)
}

/// Saves an `RgbaImage` to disk with Apple Display P3 wide-gamut metadata injected.
pub fn save_display_p3_png(img: &RgbaImage, output_path: &Path) -> Result<(), String> {
    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut raw_png_bytes = Cursor::new(Vec::new());
    img.write_to(&mut raw_png_bytes, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode raw PNG: {}", e))?;

    let p3_png_bytes = inject_display_p3_chunks(&raw_png_bytes.into_inner())?;
    std::fs::write(output_path, p3_png_bytes)
        .map_err(|e| format!("Failed to write PNG file {}: {}", output_path.display(), e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_display_p3_chunks() {
        let img = RgbaImage::new(16, 16);
        let mut raw_png = Cursor::new(Vec::new());
        img.write_to(&mut raw_png, image::ImageFormat::Png).unwrap();
        let raw_bytes = raw_png.into_inner();

        let p3_bytes = inject_display_p3_chunks(&raw_bytes).unwrap();
        assert!(p3_bytes.len() > raw_bytes.len());
        assert_eq!(&p3_bytes[0..8], b"\x89PNG\r\n\x1a\n");

        // Verify cHRM chunk is present
        assert!(p3_bytes.windows(4).any(|w| w == b"cHRM"));
        // Verify gAMA chunk is present
        assert!(p3_bytes.windows(4).any(|w| w == b"gAMA"));
    }
}
