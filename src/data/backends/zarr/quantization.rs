//! Dequantisation of geo-embeddings (the `geoemb:` convention's
//! `geoemb:quantization`, declared on the array or its groups): per pixel
//! (TESSERA: each value times its pixel's value in a sibling scale array,
//! which lacks the embedding dimension; a non-finite scale is a missing
//! pixel), signed square (AlphaEarth) or linear, with `nodata` codes missing.

use std::collections::HashMap;

use zarrs::array::ArraySubset;

use super::block::{ArrayCachedState, read_window};
use super::slice::window_subset;
use crate::data::blocks::BlockStoreError;

/// How an embedding array's stored codes become values.
#[derive(Debug, PartialEq)]
pub(super) enum Quantization {
    /// Each code times its pixel's value in a scale array.
    PerPixel(PixelScale),
    /// `(x / 127.5)² · sign(x)`, the AlphaEarth Foundations encoding.
    SignedSquare { nodata: Option<f32> },
    /// `x · scale + offset`.
    Linear {
        scale: f32,
        offset: f32,
        nodata: Option<f32>,
    },
}

impl Quantization {
    /// Turns `values`, the embedding array's window `(origin, shape)`, from
    /// codes into values; `scales` is the opened scale array of `PerPixel`.
    pub(super) fn apply(
        &self,
        scales: Option<&ArrayCachedState>,
        window: (&[usize], &[usize]),
        values: &mut [f32],
    ) -> Result<(), BlockStoreError> {
        let nodata = match *self {
            Self::PerPixel(ref q) => return scales.map_or(Ok(()), |s| q.apply(s, window, values)),
            Self::SignedSquare { nodata } | Self::Linear { nodata, .. } => nodata,
        };
        for v in values.iter_mut() {
            *v = if Some(*v) == nodata {
                f32::NAN
            } else {
                self.decode(*v)
            };
        }
        Ok(())
    }

    /// The value of code `x` (unchanged for `PerPixel`, scaled by array).
    fn decode(&self, x: f32) -> f32 {
        match *self {
            Self::SignedSquare { .. } => (x / 127.5).powi(2) * x.signum(),
            Self::Linear { scale, offset, .. } => x * scale + offset,
            Self::PerPixel(_) => x,
        }
    }

    /// The scale array a `PerPixel` quantisation reads.
    pub(super) fn scale_array(&self) -> Option<&PixelScale> {
        match self {
            Self::PerPixel(q) => Some(q),
            _ => None,
        }
    }
}

/// The scale array of an embedding array and the dimension it lacks.
#[derive(Debug, PartialEq)]
pub(super) struct PixelScale {
    pub path: String,
    pub dim: usize,
}

impl PixelScale {
    /// Scales `values`, the embedding array's window `(origin, shape)`, by
    /// that window of `scales`, the opened scale array.
    pub(super) fn apply(
        &self,
        scales: &ArrayCachedState,
        (origin, shape): (&[usize], &[usize]),
        values: &mut [f32],
    ) -> Result<(), BlockStoreError> {
        let (o, s) = self.window(origin, shape);
        let reader = scales.reader.chunk_reader();
        let pixel_scales = read_window(&scales.array, reader, &self.path, (&o, &s), false)?;
        dequantise(values, &pixel_scales, shape, self.dim);
        Ok(())
    }

    /// The scale array's `(origin, shape)` window under the embedding
    /// array's window: the same, without the embedding dimension.
    fn window(&self, origin: &[usize], shape: &[usize]) -> (Vec<usize>, Vec<usize>) {
        without(origin, self.dim)
            .zip(without(shape, self.dim))
            .unzip()
    }

    /// The scale array's window as a subset, for the browser to fetch.
    pub(super) fn subset(&self, origin: &[usize], shape: &[usize]) -> ArraySubset {
        let (origin, shape) = self.window(origin, shape);
        window_subset(&origin, &shape)
    }
}

/// The declared quantisation of an array of `shape`, from its inherited
/// attributes: only for embedding arrays (at least 2D, with a dimension of
/// the embedding's length, `geoemb:dimensions`), so a group's other arrays
/// and coordinates stay as stored.
fn declared(attributes: &HashMap<String, String>, shape: &[u64]) -> Option<serde_json::Value> {
    let len: u64 = attributes.get("geoemb:dimensions")?.parse().ok()?;
    if shape.len() < 2 || !shape.contains(&len) {
        return None;
    }
    serde_json::from_str(attributes.get("geoemb:quantization")?).ok()
}

/// A quantisation decoding each code alone (signed square or linear).
pub(super) fn value_quantization(
    attributes: &HashMap<String, String>,
    shape: &[u64],
) -> Option<Quantization> {
    let q = declared(attributes, shape)?;
    let nodata = q["nodata"].as_f64().map(|v| v as f32);
    match q["method"].as_str()? {
        "signed_square" => Some(Quantization::SignedSquare { nodata }),
        "linear" if q["scale"]["type"] == "scalar" => Some(Quantization::Linear {
            scale: q["scale"]["scale"].as_f64()? as f32,
            offset: q["scale"]["offset"].as_f64().unwrap_or(0.0) as f32,
            nodata,
        }),
        _ => None,
    }
}

/// The path of the array scaling `name` (of `shape`) per pixel.
pub(super) fn scale_path(
    attributes: &HashMap<String, String>,
    name: &str,
    shape: &[u64],
) -> Option<String> {
    let q = declared(attributes, shape)?;
    if q["method"] != "per_pixel_scale" || q["scale"]["type"] != "array" {
        return None;
    }
    let array_name = q["scale"]["array_name"].as_str()?;
    let path = match name.rsplit_once('/') {
        Some((group, _)) => format!("{group}/{array_name}"),
        None => array_name.to_string(),
    };
    (path != name).then_some(path)
}

/// The embedding dimension of an array of `shape` scaled by one of
/// `scale_shape`: of the embedding's length, leaving the scale's shape.
pub(super) fn embedding_dim(
    attributes: &HashMap<String, String>,
    shape: &[u64],
    scale_shape: &[u64],
) -> Option<usize> {
    let len: u64 = attributes.get("geoemb:dimensions")?.parse().ok()?;
    (0..shape.len())
        .rev()
        .find(|&d| shape[d] == len && without(shape, d).eq(scale_shape.iter().copied()))
}

/// `values` without its `dim`th entry.
fn without<T: Copy>(values: &[T], dim: usize) -> impl Iterator<Item = T> + '_ {
    values
        .iter()
        .enumerate()
        .filter(move |&(d, _)| d != dim)
        .map(|(_, &v)| v)
}

/// Multiplies `values`, a window of `shape`, by `scales`, the same window
/// without dimension `dim`; a non-finite scale makes its pixel's values NaN.
pub(super) fn dequantise(values: &mut [f32], scales: &[f32], shape: &[usize], dim: usize) {
    let Some(&len) = shape.get(dim) else {
        return;
    };
    let Some(inner) = shape[dim + 1..]
        .iter()
        .try_fold(1usize, |n, &d| n.checked_mul(d))
    else {
        return;
    };
    let Some(step) = len.checked_mul(inner).filter(|&s| s > 0) else {
        return;
    };
    for (block, row) in values.chunks_mut(step).zip(scales.chunks(inner)) {
        for along in block.chunks_mut(inner) {
            for (v, &s) in along.iter_mut().zip(row) {
                *v = if s.is_finite() { *v * s } else { f32::NAN };
            }
        }
    }
}
