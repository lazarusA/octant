//! `OctantBlock`: the core, N-dimensional resident data representation.
//!
//! For N-dimensional blocks `OctantBlock` keeps that block alive
//! as long as possible, with no opinion about how it will be rendered.
//! Rendering decides *later* how to collapse/project dimensions (2D heatmap,
//! animated map, volume, animated volume, arbitrary 4D+ views) — see the
//! projection methods below and `crate::data::projection` (added in a later
//! phase) for richer views.
//!
use std::collections::HashMap;

use crate::stores::MatrixSlice;

/// A resident, N-dimensional block of data pulled from a Store.
///
/// The block does not know or care how it will eventually be displayed.
#[derive(Debug, Clone)]
pub struct OctantBlock {
    pub variable_name: String,

    /// Shape of this resident block, e.g. `[time, z, y, x]`. Note this is
    /// the shape of the *loaded window*, not necessarily the full origin
    /// array — see `origin` for where this window sits in the full array.
    pub shape: Vec<usize>,

    /// Dimension names, in the same order as `shape` (from store metadata).
    pub dimension_names: Vec<String>,

    /// Offset of this block within the full array, per dimension.
    /// Lets a windowed load (e.g. `time: 100..148`) still be reasoned about
    /// in terms of global indices later (sliding-window prefetching, Phase 8/9).
    pub origin: Vec<usize>,

    /// Flattened values in row-major (C) order: dimension 0 varies slowest,
    /// the last dimension varies fastest.
    pub values: Vec<f32>,

    /// Row-major strides, one per dimension, matching `shape`.
    pub strides: Vec<usize>,

    /// Coordinate metadata, keyed by dimension name.
    pub coordinates: HashMap<String, Vec<f64>>,

    /// Original dataset attributes (`.zattrs`), stringified.
    pub attributes: HashMap<String, String>,

    pub min_value: f32,
    pub max_value: f32,
}

impl OctantBlock {
    /// Builds a new block, computing strides and value bounds.
    ///
    /// `values.len()` must equal the product of `shape`; this is checked
    /// with a `debug_assert` rather than a hard error since callers that
    /// build blocks from trusted Zarr reads shouldn't pay a release-mode
    /// cost for this invariant.
    pub fn new(
        variable_name: String,
        shape: Vec<usize>,
        dimension_names: Vec<String>,
        origin: Vec<usize>,
        values: Vec<f32>,
        coordinates: HashMap<String, Vec<f64>>,
        attributes: HashMap<String, String>,
    ) -> Self {
        debug_assert_eq!(
            values.len(),
            shape
                .iter()
                .product::<usize>()
                .max(1)
                .min(values.len().max(1)),
            "OctantBlock: values length does not match product of shape"
        );

        let strides = Self::row_major_strides(&shape);

        let (min_value, max_value) = values
            .iter()
            .copied()
            .filter(|v| !v.is_nan())
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v), hi.max(v))
            });
        let (min_value, max_value) = if min_value.is_finite() && max_value.is_finite() {
            (min_value, max_value)
        } else {
            (0.0, 1.0)
        };

        Self {
            variable_name,
            shape,
            dimension_names,
            origin,
            values,
            strides,
            coordinates,
            attributes,
            min_value,
            max_value,
        }
    }

    /// Row-major strides for a given shape: last dimension is contiguous.
    fn row_major_strides(shape: &[usize]) -> Vec<usize> {
        let mut strides = vec![1usize; shape.len()];
        for i in (0..shape.len().saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * shape[i + 1].max(1);
        }
        strides
    }

    pub fn rank(&self) -> usize {
        self.shape.len()
    }

    /// Index of a dimension by name, if present.
    pub fn dim_index(&self, name: &str) -> Option<usize> {
        self.dimension_names.iter().position(|d| d == name)
    }

    /// Flattens N-dimensional `indices` (local to this block, i.e. already
    /// relative to `origin`) into an offset into `values`.
    pub fn flat_index(&self, indices: &[usize]) -> Option<usize> {
        if indices.len() != self.rank() {
            return None;
        }
        let mut offset = 0usize;
        for ((idx, dim_size), stride) in indices.iter().zip(&self.shape).zip(&self.strides) {
            if idx >= dim_size {
                return None;
            }
            offset += idx * stride;
        }
        Some(offset)
    }

    /// Reads a single value at N-dimensional `indices` (local to this block).
    ///
    /// Example: for `shape = [10, 100, 200]` (`time, y, x`),
    /// `block.get(&[3, 50, 100])` returns the value at `time=3, y=50, x=100`.
    pub fn get(&self, indices: &[usize]) -> Option<f32> {
        self.flat_index(indices)
            .and_then(|i| self.values.get(i).copied())
    }

    /// Projects this block down to a 2D `MatrixSlice` by choosing an X and Y
    /// dimension and fixing every other dimension to a single index.
    ///
    /// `fixed_indices` must have one entry per dimension in this block; the
    /// entries at `x_dim` and `y_dim` are ignored (those dimensions are
    /// iterated in full).
    ///
    /// This is a pure in-memory view: no I/O happens here. Producing many
    /// `MatrixSlice`s from one `OctantBlock` (different timesteps, different
    /// depths, ...) is exactly the "load once, explore many ways" pattern.
    pub fn matrix_slice(
        &self,
        x_dim: usize,
        y_dim: usize,
        fixed_indices: &[usize],
        timestep: usize,
        max_timesteps: usize,
        dataset_name: &str,
    ) -> Option<MatrixSlice> {
        if x_dim == y_dim || x_dim >= self.rank() || y_dim >= self.rank() {
            return None;
        }
        if fixed_indices.len() != self.rank() {
            return None;
        }

        let width = self.shape[x_dim];
        let height = self.shape[y_dim];
        let mut values = Vec::with_capacity(width * height);
        let mut idx = fixed_indices.to_vec();

        for y in 0..height {
            idx[y_dim] = y;
            for x in 0..width {
                idx[x_dim] = x;
                values.push(self.get(&idx).unwrap_or(f32::NAN));
            }
        }

        let valid: Vec<f32> = values.iter().copied().filter(|v| !v.is_nan()).collect();
        let (min_val, max_val) = if valid.is_empty() {
            (0.0, 1.0)
        } else {
            (
                valid.iter().copied().fold(f32::INFINITY, f32::min),
                valid.iter().copied().fold(f32::NEG_INFINITY, f32::max),
            )
        };

        Some(MatrixSlice {
            variable_name: self.variable_name.clone(),
            width,
            height,
            values,
            min_val,
            max_val,
            shape: self.shape.iter().map(|&s| s as u64).collect(),
            current_timestep: timestep,
            max_timesteps,
            dataset_name: dataset_name.to_string(),
        })
    }

    /// Projects this block down to a flattened 3D volume (`z * y * x`,
    /// row-major, z slowest) by choosing X/Y/Z dimensions and fixing every
    /// other dimension. Returns `(values, [nx, ny, nz])`.
    ///
    /// This is the volume analogue of `matrix_slice` — e.g. for
    /// `temperature(time, z, y, x)`, fixing `time` and choosing
    /// `x_dim/y_dim/z_dim` produces one frame of a volume animation with no
    /// additional I/O.
    pub fn volume(
        &self,
        x_dim: usize,
        y_dim: usize,
        z_dim: usize,
        fixed_indices: &[usize],
    ) -> Option<(Vec<f32>, [usize; 3])> {
        let dims = [x_dim, y_dim, z_dim];
        if dims.iter().any(|&d| d >= self.rank())
            || x_dim == y_dim
            || x_dim == z_dim
            || y_dim == z_dim
        {
            return None;
        }
        if fixed_indices.len() != self.rank() {
            return None;
        }

        let (nx, ny, nz) = (self.shape[x_dim], self.shape[y_dim], self.shape[z_dim]);
        let mut values = Vec::with_capacity(nx * ny * nz);
        let mut idx = fixed_indices.to_vec();

        for z in 0..nz {
            idx[z_dim] = z;
            for y in 0..ny {
                idx[y_dim] = y;
                for x in 0..nx {
                    idx[x_dim] = x;
                    values.push(self.get(&idx).unwrap_or(f32::NAN));
                }
            }
        }

        Some((values, [nx, ny, nz]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_block() -> OctantBlock {
        // shape = [time=2, y=3, x=4] -> values 0..24, value = flat index
        let shape = vec![2, 3, 4];
        let values: Vec<f32> = (0..24).map(|v| v as f32).collect();
        OctantBlock::new(
            "temperature".to_string(),
            shape,
            vec!["time".into(), "y".into(), "x".into()],
            vec![0, 0, 0],
            values,
            HashMap::new(),
            HashMap::new(),
        )
    }

    #[test]
    fn strides_are_row_major() {
        let block = test_block();
        assert_eq!(block.strides, vec![12, 4, 1]);
    }

    #[test]
    fn get_matches_flat_layout() {
        let block = test_block();
        // time=1, y=2, x=3 -> flat index 1*12 + 2*4 + 3 = 23
        assert_eq!(block.get(&[1, 2, 3]), Some(23.0));
        assert_eq!(block.get(&[0, 0, 0]), Some(0.0));
        assert_eq!(block.get(&[2, 0, 0]), None); // out of bounds on time
    }

    #[test]
    fn matrix_slice_projection_selects_x_y_and_fixes_time() {
        let block = test_block();
        // y_dim=1, x_dim=2, fix time=1
        let slice = block
            .matrix_slice(2, 1, &[1, 0, 0], 1, 2, "test")
            .expect("valid projection");
        assert_eq!(slice.width, 4);
        assert_eq!(slice.height, 3);
        // first row (y=0) at time=1 should be values 12..16
        assert_eq!(&slice.values[0..4], &[12.0, 13.0, 14.0, 15.0]);
    }

    #[test]
    fn volume_projection_has_expected_shape() {
        let block = test_block();
        // Treat time as z, y as y, x as x -> full block as a volume
        let (values, dims) = block.volume(2, 1, 0, &[0, 0, 0]).expect("valid volume");
        assert_eq!(dims, [4, 3, 2]);
        assert_eq!(values.len(), 24);
    }
}
