//! Self-contained predictor reversal routines for TIFF buffers.

use async_tiff::tags::Predictor;

/// Reverse pre-compression predictors (Predictor 2 horizontal diff, Predictor 3 floating point).
pub fn unpredict_buffer(
    mut buffer: Vec<u8>,
    predictor: Predictor,
    samples: usize,
    bits_per_sample: u16,
    width: usize,
    is_big_endian: bool,
) -> Result<Vec<u8>, String> {
    match predictor {
        Predictor::None => {
            if is_big_endian {
                swap_endianness(&mut buffer, bits_per_sample);
            }
            Ok(buffer)
        }
        Predictor::Horizontal => {
            if is_big_endian {
                swap_endianness(&mut buffer, bits_per_sample);
            }
            unpredict_horizontal(&mut buffer, samples, bits_per_sample, width)?;
            Ok(buffer)
        }
        Predictor::FloatingPoint => unpredict_float(buffer, samples, bits_per_sample, width),
        _ => Ok(buffer),
    }
}

fn swap_endianness(buf: &mut [u8], bits: u16) {
    match bits {
        16 => {
            for chunk in buf.chunks_exact_mut(2) {
                chunk.swap(0, 1);
            }
        }
        32 => {
            for chunk in buf.chunks_exact_mut(4) {
                chunk.reverse();
            }
        }
        64 => {
            for chunk in buf.chunks_exact_mut(8) {
                chunk.reverse();
            }
        }
        _ => {}
    }
}

fn unpredict_horizontal(
    buf: &mut [u8],
    samples: usize,
    bits: u16,
    width: usize,
) -> Result<(), String> {
    let bytes_per_sample = (bits as usize).div_ceil(8);
    let row_stride = width * samples * bytes_per_sample;
    if row_stride == 0 {
        return Ok(());
    }

    for row in buf.chunks_mut(row_stride) {
        match bits {
            0..=8 => {
                for i in samples..row.len() {
                    row[i] = row[i].wrapping_add(row[i - samples]);
                }
            }
            9..=16 => {
                let step = samples * 2;
                for i in (step..row.len()).step_by(2) {
                    let prev = u16::from_ne_bytes([row[i - step], row[i - step + 1]]);
                    let curr = u16::from_ne_bytes([row[i], row[i + 1]]);
                    let sum = curr.wrapping_add(prev).to_ne_bytes();
                    row[i] = sum[0];
                    row[i + 1] = sum[1];
                }
            }
            17..=32 => {
                let step = samples * 4;
                for i in (step..row.len()).step_by(4) {
                    let prev = u32::from_ne_bytes(
                        row[i - step..i - step + 4].try_into().unwrap_or_default(),
                    );
                    let curr = u32::from_ne_bytes(row[i..i + 4].try_into().unwrap_or_default());
                    let sum = curr.wrapping_add(prev).to_ne_bytes();
                    row[i..i + 4].copy_from_slice(&sum);
                }
            }
            33..=64 => {
                let step = samples * 8;
                for i in (step..row.len()).step_by(8) {
                    let prev = u64::from_ne_bytes(
                        row[i - step..i - step + 8].try_into().unwrap_or_default(),
                    );
                    let curr = u64::from_ne_bytes(row[i..i + 8].try_into().unwrap_or_default());
                    let sum = curr.wrapping_add(prev).to_ne_bytes();
                    row[i..i + 8].copy_from_slice(&sum);
                }
            }
            _ => return Err(format!("Unsupported bits_per_sample for predictor: {bits}")),
        }
    }
    Ok(())
}

fn unpredict_float(
    mut input: Vec<u8>,
    samples: usize,
    bits: u16,
    width: usize,
) -> Result<Vec<u8>, String> {
    let bytes_per_sample = (bits as usize) / 8;
    let row_stride = width * samples * bytes_per_sample;
    if row_stride == 0 {
        return Ok(input);
    }

    let mut output = vec![0u8; input.len()];
    for (in_row, out_row) in input
        .chunks_mut(row_stride)
        .zip(output.chunks_mut(row_stride))
    {
        for i in samples..in_row.len() {
            in_row[i] = in_row[i].wrapping_add(in_row[i - samples]);
        }
        let plane_len = in_row.len() / bytes_per_sample;
        match bytes_per_sample {
            2 => {
                for (i, chunk) in out_row.chunks_exact_mut(2).enumerate() {
                    let bytes =
                        u16::from_be_bytes([in_row[i], in_row[plane_len + i]]).to_ne_bytes();
                    chunk.copy_from_slice(&bytes);
                }
            }
            4 => {
                for (i, chunk) in out_row.chunks_exact_mut(4).enumerate() {
                    let bytes = u32::from_be_bytes([
                        in_row[i],
                        in_row[plane_len + i],
                        in_row[plane_len * 2 + i],
                        in_row[plane_len * 3 + i],
                    ])
                    .to_ne_bytes();
                    chunk.copy_from_slice(&bytes);
                }
            }
            8 => {
                for (i, chunk) in out_row.chunks_exact_mut(8).enumerate() {
                    let b: [u8; 8] = std::array::from_fn(|idx| in_row[plane_len * idx + i]);
                    let bytes = u64::from_be_bytes(b).to_ne_bytes();
                    chunk.copy_from_slice(&bytes);
                }
            }
            _ => return Err(format!("Unsupported float predictor bits: {bits}")),
        }
    }
    Ok(output)
}
