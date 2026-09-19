//! Sample unpacking routines converting raw byte slices into typed `f32` vectors.

use async_tiff::tags::{PhotometricInterpretation, SampleFormat};

/// Unpack raw decompressed/unpredicted buffer into `f32` values with row byte alignment.
pub fn unpack_samples_to_f32(
    buffer: &[u8],
    bits_per_sample: u16,
    sample_format: SampleFormat,
    photometric: PhotometricInterpretation,
    samples_per_row: usize,
    num_rows: usize,
) -> Vec<f32> {
    let total_elements = samples_per_row * num_rows;
    let mut out = Vec::with_capacity(total_elements);

    match (sample_format, bits_per_sample) {
        (SampleFormat::Uint, 1) => {
            let is_white_zero = photometric == PhotometricInterpretation::WhiteIsZero;
            let row_bytes_len = samples_per_row.div_ceil(8);
            for row_idx in 0..num_rows {
                let r_start = row_idx * row_bytes_len;
                let r_end = (r_start + row_bytes_len).min(buffer.len());
                if r_start >= buffer.len() {
                    break;
                }
                let row_bytes = &buffer[r_start..r_end];
                let mut row_count = 0;
                for &byte in row_bytes {
                    for bit in (0..8).rev() {
                        if row_count >= samples_per_row {
                            break;
                        }
                        let bit_val = (byte >> bit) & 1;
                        let val = if is_white_zero {
                            if bit_val == 0 { 1.0 } else { 0.0 }
                        } else {
                            bit_val as f32
                        };
                        out.push(val);
                        row_count += 1;
                    }
                }
                while row_count < samples_per_row {
                    out.push(if is_white_zero { 1.0 } else { 0.0 });
                    row_count += 1;
                }
            }
        }
        (SampleFormat::Uint, 4) => {
            let row_bytes_len = samples_per_row.div_ceil(2);
            for row_idx in 0..num_rows {
                let r_start = row_idx * row_bytes_len;
                let r_end = (r_start + row_bytes_len).min(buffer.len());
                if r_start >= buffer.len() {
                    break;
                }
                let row_bytes = &buffer[r_start..r_end];
                let mut row_count = 0;
                for &byte in row_bytes {
                    if row_count < samples_per_row {
                        out.push((byte >> 4) as f32);
                        row_count += 1;
                    }
                    if row_count < samples_per_row {
                        out.push((byte & 0x0F) as f32);
                        row_count += 1;
                    }
                }
                while row_count < samples_per_row {
                    out.push(0.0);
                    row_count += 1;
                }
            }
        }
        (SampleFormat::Uint, 8) => {
            let is_white_zero = photometric == PhotometricInterpretation::WhiteIsZero;
            for &byte in buffer.iter().take(total_elements) {
                let val = if is_white_zero {
                    (255 - byte) as f32
                } else {
                    byte as f32
                };
                out.push(val);
            }
        }
        (SampleFormat::Int, 8) => {
            for &byte in buffer.iter().take(total_elements) {
                out.push(byte as i8 as f32);
            }
        }
        (SampleFormat::Uint, 12) => {
            let row_bytes_len = (samples_per_row * 12).div_ceil(8);
            for row_idx in 0..num_rows {
                let r_start = row_idx * row_bytes_len;
                let r_end = (r_start + row_bytes_len).min(buffer.len());
                if r_start >= buffer.len() {
                    break;
                }
                let row_bytes = &buffer[r_start..r_end];
                let mut row_count = 0;
                for chunk in row_bytes.chunks(3) {
                    if row_count >= samples_per_row {
                        break;
                    }
                    if chunk.len() >= 2 {
                        let b0 = chunk[0] as u16;
                        let b1 = chunk[1] as u16;
                        out.push(((b0 << 4) | (b1 >> 4)) as f32);
                        row_count += 1;
                    }
                    if row_count < samples_per_row && chunk.len() == 3 {
                        let b1 = chunk[1] as u16;
                        let b2 = chunk[2] as u16;
                        out.push((((b1 & 0x0F) << 8) | b2) as f32);
                        row_count += 1;
                    }
                }
                while row_count < samples_per_row {
                    out.push(0.0);
                    row_count += 1;
                }
            }
        }
        (SampleFormat::Uint, 16) => {
            for chunk in buffer.chunks_exact(2).take(total_elements) {
                let val = u16::from_ne_bytes([chunk[0], chunk[1]]);
                out.push(val as f32);
            }
        }
        (SampleFormat::Int, 16) => {
            for chunk in buffer.chunks_exact(2).take(total_elements) {
                let val = i16::from_ne_bytes([chunk[0], chunk[1]]);
                out.push(val as f32);
            }
        }
        (SampleFormat::Float, 16) => {
            for chunk in buffer.chunks_exact(2).take(total_elements) {
                let bits = u16::from_ne_bytes([chunk[0], chunk[1]]);
                let f16_val = half::f16::from_bits(bits);
                out.push(f16_val.to_f32());
            }
        }
        (SampleFormat::Uint, 32) => {
            for chunk in buffer.chunks_exact(4).take(total_elements) {
                let val = u32::from_ne_bytes(chunk.try_into().unwrap_or_default());
                out.push(val as f32);
            }
        }
        (SampleFormat::Int, 32) => {
            for chunk in buffer.chunks_exact(4).take(total_elements) {
                let val = i32::from_ne_bytes(chunk.try_into().unwrap_or_default());
                out.push(val as f32);
            }
        }
        (SampleFormat::Float, 32) => {
            for chunk in buffer.chunks_exact(4).take(total_elements) {
                let val = f32::from_ne_bytes(chunk.try_into().unwrap_or_default());
                out.push(val);
            }
        }
        (SampleFormat::Float, 64) => {
            for chunk in buffer.chunks_exact(8).take(total_elements) {
                let val = f64::from_ne_bytes(chunk.try_into().unwrap_or_default());
                out.push(val as f32);
            }
        }
        (SampleFormat::Uint, 64) => {
            for chunk in buffer.chunks_exact(8).take(total_elements) {
                let val = u64::from_ne_bytes(chunk.try_into().unwrap_or_default());
                out.push(val as f32);
            }
        }
        (SampleFormat::Int, 64) => {
            for chunk in buffer.chunks_exact(8).take(total_elements) {
                let val = i64::from_ne_bytes(chunk.try_into().unwrap_or_default());
                out.push(val as f32);
            }
        }
        _ => {
            for &byte in buffer.iter().take(total_elements) {
                out.push(byte as f32);
            }
        }
    }

    if out.len() < total_elements {
        out.resize(total_elements, f32::NAN);
    }

    out
}
