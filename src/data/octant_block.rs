//! N-dimensional resident data representation.
//!
//! `OctantBlock` is deliberately independent of the storage format.
//! A block can originate from Zarr, NetCDF, GeoTIFF, Icechunk, etc.
//! Once resident, rendering code does not need to know where it came from.

use std::collections::HashMap;

use crate::stores::MatrixSlice;

#[derive(Debug, Clone)]
pub struct OctantBlock {
    pub variable_name: String,

    /// Shape of the loaded window, not necessarily the full dataset.
    pub shape: Vec<usize>,

    /// Dimension names corresponding to `shape`.
    pub dimension_names: Vec<String>,

    /// Global origin of this block inside the source array.
    pub origin: Vec<usize>,

    /// Row-major values.
    pub values: Vec<f32>,

    /// Row-major strides.
    pub strides: Vec<usize>,

    /// Coordinate metadata keyed by dimension name.
    pub coordinates: HashMap<String, Vec<f64>>,

    /// Source attributes.
    pub attributes: HashMap<String, String>,

    pub min_value: f32,
    pub max_value: f32,
}

impl OctantBlock {
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
            shape.iter().copied().product::<usize>(),
            "OctantBlock: values length does not match shape"
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

    fn row_major_strides(shape: &[usize]) -> Vec<usize> {
        let mut strides = vec![1; shape.len()];

        for i in (0..shape.len().saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * shape[i + 1].max(1);
        }

        strides
    }

    pub fn rank(&self) -> usize {
        self.shape.len()
    }

    pub fn dim_index(&self, name: &str) -> Option<usize> {
        self.dimension_names.iter().position(|d| d == name)
    }

    pub fn flat_index(&self, indices: &[usize]) -> Option<usize> {
        if indices.len() != self.rank() {
            return None;
        }

        let mut offset = 0usize;

        for ((idx, size), stride) in indices.iter().zip(&self.shape).zip(&self.strides) {
            if *idx >= *size {
                return None;
            }

            offset += idx * stride;
        }

        Some(offset)
    }

    pub fn get(&self, indices: &[usize]) -> Option<f32> {
        self.flat_index(indices)
            .and_then(|i| self.values.get(i).copied())
    }

    pub fn matrix_slice(
        &self,
        x_dim: usize,
        y_dim: usize,
        fixed_indices: &[usize],
        timestep: usize,
        max_timesteps: usize,
        dataset_name: &str,
    ) -> Option<MatrixSlice> {
        if x_dim == y_dim
            || x_dim >= self.rank()
            || y_dim >= self.rank()
            || fixed_indices.len() != self.rank()
        {
            return None;
        }

        let width = self.shape[x_dim];
        let height = self.shape[y_dim];

        let mut values = Vec::with_capacity(width * height);
        let mut indices = fixed_indices.to_vec();

        for y in 0..height {
            indices[y_dim] = y;

            for x in 0..width {
                indices[x_dim] = x;
                values.push(self.get(&indices).unwrap_or(f32::NAN));
            }
        }

        let (min_val, max_val) = values
            .iter()
            .copied()
            .filter(|v| !v.is_nan())
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v), hi.max(v))
            });

        let (min_val, max_val) = if min_val.is_finite() && max_val.is_finite() {
            (min_val, max_val)
        } else {
            (0.0, 1.0)
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

    pub fn volume(
        &self,
        x_dim: usize,
        y_dim: usize,
        z_dim: usize,
        fixed_indices: &[usize],
    ) -> Option<(Vec<f32>, [usize; 3])> {
        if x_dim >= self.rank()
            || y_dim >= self.rank()
            || z_dim >= self.rank()
            || x_dim == y_dim
            || x_dim == z_dim
            || y_dim == z_dim
            || fixed_indices.len() != self.rank()
        {
            return None;
        }

        let nx = self.shape[x_dim];
        let ny = self.shape[y_dim];
        let nz = self.shape[z_dim];

        let mut values = Vec::with_capacity(nx * ny * nz);
        let mut indices = fixed_indices.to_vec();

        for z in 0..nz {
            indices[z_dim] = z;

            for y in 0..ny {
                indices[y_dim] = y;

                for x in 0..nx {
                    indices[x_dim] = x;
                    values.push(self.get(&indices).unwrap_or(f32::NAN));
                }
            }
        }

        Some((values, [nx, ny, nz]))
    }

    pub fn bytes_size(&self) -> usize {
        self.values.len() * std::mem::size_of::<f32>()
    }
}
