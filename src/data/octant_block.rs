//! N-dimensional resident data representation.
//!
//! `OctantBlock` is deliberately independent of the storage format.
//! A block can originate from Zarr, NetCDF, GeoTIFF, Icechunk, etc.
//! Once resident, rendering code does not need to know where it came from.

use std::collections::HashMap;

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

    pub fn slice_2d(
        &self,
        x_dim: usize,
        y_dim: usize,
        fixed_indices: &[usize],
        max_timesteps: usize,
        dataset_name: &str,
    ) -> Option<crate::data::matrix_data::MatrixData> {
        if x_dim == y_dim
            || x_dim >= self.rank()
            || y_dim >= self.rank()
            || fixed_indices.len() != self.rank()
        {
            return None;
        }

        let width = self.shape[x_dim];
        let height = self.shape[y_dim];

        let stride_x = self.strides[x_dim];
        let stride_y = self.strides[y_dim];

        let mut base_offset = 0usize;
        for (i, &fixed_idx) in fixed_indices.iter().enumerate().take(self.rank()) {
            if i != x_dim && i != y_dim {
                let idx = fixed_idx.min(self.shape[i].saturating_sub(1));
                base_offset += idx * self.strides[i];
            }
        }

        let mut values = Vec::with_capacity(width * height);
        if stride_x == 1 {
            for y in 0..height {
                let row_start = base_offset + y * stride_y;
                let row_end = row_start + width;
                if row_end <= self.values.len() {
                    values.extend_from_slice(&self.values[row_start..row_end]);
                } else {
                    for x in 0..width {
                        values.push(self.values.get(row_start + x).copied().unwrap_or(f32::NAN));
                    }
                }
            }
        } else {
            for y in 0..height {
                let row_start = base_offset + y * stride_y;
                for x in 0..width {
                    let idx = row_start + x * stride_x;
                    values.push(self.values.get(idx).copied().unwrap_or(f32::NAN));
                }
            }
        }

        let (min_val, max_val) = if self.min_value.is_finite() && self.max_value.is_finite() {
            (self.min_value, self.max_value)
        } else {
            let (lo, hi) = values
                .iter()
                .copied()
                .filter(|v| !v.is_nan())
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
                    (lo.min(v), hi.max(v))
                });
            if lo.is_finite() && hi.is_finite() {
                (lo, hi)
            } else {
                (0.0, 1.0)
            }
        };

        Some(crate::data::matrix_data::MatrixData::new(
            width,
            height,
            values,
            min_val,
            max_val,
            dataset_name.to_string(),
            max_timesteps,
        ))
    }

    pub fn volume(
        &self,
        x_dim: usize,
        y_dim: usize,
        z_dim: usize,
        fixed_indices: &[usize],
        dataset_name: &str,
    ) -> Option<crate::data::VolumeData> {
        if x_dim >= self.rank() || y_dim >= self.rank() || fixed_indices.len() != self.rank() {
            return None;
        }

        let nx = self.shape[x_dim];
        let ny = self.shape[y_dim];
        let (nz, has_z) = if z_dim < self.rank() && z_dim != x_dim && z_dim != y_dim {
            (self.shape[z_dim], true)
        } else {
            (1, false)
        };

        if x_dim == y_dim {
            return None;
        }

        // Limit nz so that nx * ny * eff_nz <= MAX_GPU_STORAGE_BUFFER_ELEMENTS
        let slice_elements = nx.saturating_mul(ny).max(1);
        let max_z = (crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS / slice_elements)
            .clamp(1, nz);
        let eff_nz = nz.min(max_z);

        // Fast path: if 3D array matching shape [nz, ny, nx] exactly and fits within GPU buffer limit
        if self.rank() == 3
            && z_dim == 0
            && y_dim == 1
            && x_dim == 2
            && self.values.len() == nx * ny * nz
            && nz == eff_nz
        {
            let (min_val, max_val) = self
                .values
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

            return Some(crate::data::VolumeData::new(
                nx,
                ny,
                nz,
                self.values.clone(),
                min_val,
                max_val,
                dataset_name.to_string(),
            ));
        }

        let stride_x = self.strides[x_dim];
        let stride_y = self.strides[y_dim];
        let stride_z = if has_z { self.strides[z_dim] } else { 0 };

        let mut base_offset = 0usize;
        for (i, &fixed_idx) in fixed_indices.iter().enumerate().take(self.rank()) {
            if i != x_dim && i != y_dim && (!has_z || i != z_dim) {
                let idx = fixed_idx.min(self.shape[i].saturating_sub(1));
                base_offset += idx * self.strides[i];
            }
        }

        let mut values = Vec::with_capacity(nx * ny * eff_nz);
        if stride_x == 1 {
            for z in 0..eff_nz {
                let z_offset = base_offset + z * stride_z;
                for y in 0..ny {
                    let row_start = z_offset + y * stride_y;
                    let row_end = row_start + nx;
                    if row_end <= self.values.len() {
                        values.extend_from_slice(&self.values[row_start..row_end]);
                    } else {
                        for x in 0..nx {
                            values
                                .push(self.values.get(row_start + x).copied().unwrap_or(f32::NAN));
                        }
                    }
                }
            }
        } else {
            for z in 0..eff_nz {
                let z_offset = base_offset + z * stride_z;
                for y in 0..ny {
                    let row_start = z_offset + y * stride_y;
                    for x in 0..nx {
                        let idx = row_start + x * stride_x;
                        values.push(self.values.get(idx).copied().unwrap_or(f32::NAN));
                    }
                }
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

        Some(crate::data::VolumeData::new(
            nx,
            ny,
            eff_nz,
            values,
            min_val,
            max_val,
            dataset_name.to_string(),
        ))
    }

    pub fn bytes_size(&self) -> usize {
        self.values.len() * std::mem::size_of::<f32>()
    }
}
