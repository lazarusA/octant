//! N-dimensional resident data representation.
//!
//! `OctantBlock` is deliberately independent of the storage format.
//! A block can originate from Zarr, NetCDF, GeoTIFF, Icechunk, etc.
//! Once resident, rendering code does not need to know where it came from.

use std::collections::HashMap;
use std::sync::Arc;

/// 2D curvilinear coordinate array (e.g. `lon(y, x)` or `lat(y, x)`).
#[derive(Debug, Clone, PartialEq)]
pub struct CurvilinearCoord2D {
    pub values: Arc<[f32]>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct OctantBlock {
    pub variable_name: String,

    /// Shape of the loaded window, not necessarily the full dataset.
    pub shape: Vec<usize>,

    /// Dimension names corresponding to `shape`.
    pub dimension_names: Vec<String>,

    /// Global origin of this block inside the source array.
    pub origin: Vec<usize>,

    /// Row-major values wrapped in Arc for O(1) cloning and zero-copy sharing.
    pub values: Arc<[f32]>,

    /// Row-major strides.
    pub strides: Vec<usize>,

    /// Coordinate metadata keyed by dimension name.
    pub coordinates: HashMap<String, Vec<f64>>,

    /// 2D curvilinear coordinate arrays keyed by variable name (e.g. "nav_lon", "nav_lat", "lon", "lat").
    pub curvilinear_coordinates: HashMap<String, CurvilinearCoord2D>,

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
        values: impl Into<Arc<[f32]>>,
        coordinates: HashMap<String, Vec<f64>>,
        attributes: HashMap<String, String>,
    ) -> Self {
        let values: Arc<[f32]> = values.into();
        debug_assert_eq!(
            values.len(),
            shape
                .iter()
                .copied()
                .try_fold(1usize, |acc, d| acc.checked_mul(d))
                .unwrap_or(usize::MAX),
            "OctantBlock: values length does not match shape"
        );

        let strides = Self::row_major_strides(&shape);
        let (min_value, max_value) = crate::utils::compute_finite_min_max(&values);

        Self {
            variable_name,
            shape,
            dimension_names,
            origin,
            values,
            strides,
            coordinates,
            curvilinear_coordinates: HashMap::new(),
            attributes,
            min_value,
            max_value,
        }
    }

    pub fn with_curvilinear_coordinates(
        mut self,
        curvilinear_coordinates: HashMap<String, CurvilinearCoord2D>,
    ) -> Self {
        self.curvilinear_coordinates = curvilinear_coordinates;
        self
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
        compute_bounds: bool,
    ) -> Option<crate::data::matrix_data::MatrixData> {
        let x_len = self.shape.get(x_dim).copied().unwrap_or(0);
        let y_len = self.shape.get(y_dim).copied().unwrap_or(0);
        self.slice_2d_with_ranges(
            x_dim,
            y_dim,
            (0, x_len),
            (0, y_len),
            fixed_indices,
            max_timesteps,
            dataset_name,
            compute_bounds,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn slice_2d_with_ranges(
        &self,
        x_dim: usize,
        y_dim: usize,
        x_range: (usize, usize),
        y_range: (usize, usize),
        fixed_indices: &[usize],
        max_timesteps: usize,
        dataset_name: &str,
        compute_bounds: bool,
    ) -> Option<crate::data::matrix_data::MatrixData> {
        super::slicing::slice_2d_with_ranges(
            self,
            x_dim,
            y_dim,
            x_range,
            y_range,
            fixed_indices,
            max_timesteps,
            dataset_name,
            compute_bounds,
        )
    }

    pub fn volume(
        &self,
        x_dim: usize,
        y_dim: usize,
        z_dim: usize,
        fixed_indices: &[usize],
        dataset_name: &str,
        compute_bounds: bool,
    ) -> Option<crate::data::VolumeData> {
        let has_z = z_dim < self.rank();
        let x_len = self.shape.get(x_dim).copied().unwrap_or(0);
        let y_len = self.shape.get(y_dim).copied().unwrap_or(0);
        let z_len = if has_z {
            self.shape.get(z_dim).copied().unwrap_or(1)
        } else {
            1
        };

        self.volume_with_ranges(
            x_dim,
            y_dim,
            z_dim,
            (0, x_len),
            (0, y_len),
            (0, z_len),
            fixed_indices,
            dataset_name,
            compute_bounds,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn volume_with_ranges(
        &self,
        x_dim: usize,
        y_dim: usize,
        z_dim: usize,
        x_range: (usize, usize),
        y_range: (usize, usize),
        z_range: (usize, usize),
        fixed_indices: &[usize],
        dataset_name: &str,
        compute_bounds: bool,
    ) -> Option<crate::data::VolumeData> {
        super::slicing::volume_with_ranges(
            self,
            x_dim,
            y_dim,
            z_dim,
            x_range,
            y_range,
            z_range,
            fixed_indices,
            dataset_name,
            compute_bounds,
        )
    }

    pub fn bytes_size(&self) -> usize {
        self.values.len() * std::mem::size_of::<f32>()
    }
}
