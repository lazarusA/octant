//! Predictor transformations (Horizontal and Floating-point) for TIFF decompression.

use async_tiff::reader::Endianness;
use async_tiff::tags::Predictor;

/// Reverse pre-compression predictors and normalize byte endianness.
pub fn unpredict_buffer(
    mut buf: Vec<u8>,
    predictor: Predictor,
    samples: usize,
    bits: u16,
    width: usize,
    endianness: Endianness,
) -> Result<Vec<u8>, String> {
    if endianness == Endianness::BigEndian {
        match bits {
            16 => buf
                .as_chunks_mut::<2>()
                .0
                .iter_mut()
                .for_each(|c| c.swap(0, 1)),
            32 => buf
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .for_each(|c| c.reverse()),
            64 => buf
                .as_chunks_mut::<8>()
                .0
                .iter_mut()
                .for_each(|c| c.reverse()),
            _ => {}
        }
    }
    match predictor {
        Predictor::Horizontal => unpredict_horizontal(&mut buf, samples, bits, width)?,
        Predictor::FloatingPoint => return unpredict_float(buf, samples, bits, width),
        _ => {}
    }
    Ok(buf)
}

macro_rules! unpred_diff {
    ($T:ident, $row:expr, $samples:expr) => {{
        let sz = std::mem::size_of::<$T>();
        let step = $samples * sz;
        // A truncated row's partial last sample is left as it is.
        let whole = $row.len() - $row.len() % sz;
        for i in (step..whole).step_by(sz) {
            let prev =
                $T::from_ne_bytes($row[i - step..i - step + sz].try_into().unwrap_or_default());
            let sum = $T::from_ne_bytes($row[i..i + sz].try_into().unwrap_or_default())
                .wrapping_add(prev)
                .to_ne_bytes();
            $row[i..i + sz].copy_from_slice(&sum);
        }
    }};
}

fn unpredict_horizontal(
    buf: &mut [u8],
    samples: usize,
    bits: u16,
    width: usize,
) -> Result<(), String> {
    let bytes_per_sample = (bits as usize).div_ceil(8);
    let row_stride = width.saturating_mul(samples * bytes_per_sample);
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
            9..=16 => unpred_diff!(u16, row, samples),
            17..=32 => unpred_diff!(u32, row, samples),
            33..=64 => unpred_diff!(u64, row, samples),
            _ => return Err(format!("Unsupported bits for horizontal predictor: {bits}")),
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
    let bps = (bits as usize) / 8;
    let row_stride = width.saturating_mul(samples * bps);
    if row_stride == 0 {
        return Ok(input);
    }
    if !matches!(bps, 2 | 4 | 8) {
        return Err(format!("Unsupported float predictor bits: {bits}"));
    }
    let mut output = vec![0u8; input.len()];
    for (in_row, out_row) in input
        .chunks_mut(row_stride)
        .zip(output.chunks_mut(row_stride))
    {
        for i in samples..in_row.len() {
            in_row[i] = in_row[i].wrapping_add(in_row[i - samples]);
        }
        let plane_len = in_row.len() / bps;
        for (i, chunk) in out_row.chunks_exact_mut(bps).enumerate() {
            for (b, byte) in chunk.iter_mut().enumerate() {
                *byte = in_row[plane_len * b + i];
            }
            #[cfg(target_endian = "little")]
            chunk.reverse();
        }
    }
    Ok(output)
}
