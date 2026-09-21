//! 2D matrix data container and layout definitions for rendering.

use crate::data::coordinates::CoordinateGrid;

/// Spatial layout and dimension structure of sliced matrix data.
#[derive(Clone, Debug, PartialEq)]
pub enum SpatialLayout {
    /// Regular / Curvilinear 2D tensor of dimensions width x height
    Structured2D { width: usize, height: usize },
    /// Discrete global grid (HEALPix, ICON, MPAS) consisting of discrete cells
    DiscreteGlobal { num_cells: usize },
    /// Unstructured triangle / polygon mesh
    UnstructuredMesh {
        num_faces: usize,
        num_vertices: usize,
    },
}

#[derive(Clone, Debug)]
pub struct MatrixData {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
    pub min_val: f32,
    pub max_val: f32,
    pub dataset_name: String,
    pub max_timesteps: usize,
    pub unique_values: Option<Vec<f32>>,
    pub grid: CoordinateGrid,
}

impl MatrixData {
    pub fn new(
        width: usize,
        height: usize,
        values: Vec<f32>,
        min_val: f32,
        max_val: f32,
        dataset_name: String,
        max_timesteps: usize,
    ) -> Self {
        Self::new_with_grid(
            width,
            height,
            values,
            min_val,
            max_val,
            dataset_name,
            max_timesteps,
            CoordinateGrid::GlobalRegular,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_grid(
        width: usize,
        height: usize,
        values: Vec<f32>,
        min_val: f32,
        max_val: f32,
        dataset_name: String,
        max_timesteps: usize,
        grid: CoordinateGrid,
    ) -> Self {
        let unique_values = Self::compute_unique_values(&values);
        Self {
            width,
            height,
            values,
            min_val,
            max_val,
            dataset_name,
            max_timesteps,
            unique_values,
            grid,
        }
    }

    pub fn with_grid(mut self, grid: CoordinateGrid) -> Self {
        self.grid = grid;
        self
    }

    /// Returns the spatial layout and dimension structure of this matrix.
    pub fn layout(&self) -> SpatialLayout {
        if self.grid.spatial_rank() == 1 {
            SpatialLayout::DiscreteGlobal {
                num_cells: if self.height == 1 {
                    self.width
                } else {
                    self.width * self.height
                },
            }
        } else {
            SpatialLayout::Structured2D {
                width: self.width,
                height: self.height,
            }
        }
    }

    /// Generates a random 2D scalar field for visualization
    pub fn create_random_matrix(
        width: usize,
        height: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (raw_data, min_v, max_v) =
            crate::data::procedural::generate_procedural_matrix(width, height, 0);

        Ok(Self::new(
            width,
            height,
            raw_data,
            min_v,
            max_v,
            format!("Random Matrix ({width}x{height})"),
            1,
        ))
    }

    /// Fast strided unique value detector (samples up to 2000 elements with early exit when > 20 unique values).
    pub fn compute_unique_values(values: &[f32]) -> Option<Vec<f32>> {
        if values.is_empty() {
            return None;
        }

        let step = (values.len() / 2000).max(1);
        let mut unique: Vec<f32> = Vec::with_capacity(24);

        for &val in values.iter().step_by(step) {
            if val.is_nan() || val.is_infinite() {
                continue;
            }
            if !unique.iter().any(|&u| (u - val).abs() < 1e-5) {
                unique.push(val);
                if unique.len() > 20 {
                    return None; // Exceeds 20 categories -> Continuous field
                }
            }
        }

        if unique.len() >= 2 {
            unique.sort_by(f32::total_cmp);
            Some(unique)
        } else {
            None
        }
    }

    /// Returns pre-cached unique values in O(1) constant time per frame without allocations.
    pub fn detect_unique_values(&self) -> Option<&[f32]> {
        self.unique_values.as_deref()
    }

    /// Extracts a 1D line profile along dimension axis (`dim_axis`: 0 = along X/width, 1 = along Y/height).
    pub fn extract_1d_line_profile(&self, dim_axis: usize, slice_idx: usize) -> Vec<f32> {
        if self.values.is_empty() || self.width == 0 || self.height == 0 {
            return Vec::new();
        }

        if dim_axis == 0 {
            // Along X (width): extract row at `slice_idx`
            let row = slice_idx.min(self.height.saturating_sub(1));
            let start = (row * self.width).min(self.values.len());
            let end = (start + self.width).min(self.values.len());
            self.values[start..end].to_vec()
        } else {
            // Along Y (height): extract column at `slice_idx`
            let col = slice_idx.min(self.width.saturating_sub(1));
            (0..self.height)
                .map(|y| {
                    let idx = y * self.width + col;
                    self.values.get(idx).copied().unwrap_or(f32::NAN)
                })
                .collect()
        }
    }

    /// Extracts all 1D line profiles along dimension axis (`dim_axis`: 0 = all rows along X, 1 = all columns along Y).
    pub fn extract_all_1d_line_profiles(&self, dim_axis: usize) -> Vec<Vec<f32>> {
        if dim_axis == 0 {
            (0..self.height)
                .map(|y| self.extract_1d_line_profile(0, y))
                .collect()
        } else {
            (0..self.width)
                .map(|x| self.extract_1d_line_profile(1, x))
                .collect()
        }
    }

    /// Extracts all valid finite line profiles along dimension axis flattened into a contiguous payload.
    /// Returns `(payload, profile_length, valid_lines_count)`.
    pub fn extract_all_lines_payload(&self, dim_axis: usize) -> (Vec<f32>, u32, u32) {
        if dim_axis == 0 {
            let profile_length = self.width;
            let line_count = self.height;
            let mut payload = Vec::with_capacity(profile_length * line_count);
            let mut valid_lines = 0u32;
            for row in 0..line_count {
                let start = row * profile_length;
                let end = (start + profile_length).min(self.values.len());
                let row_slice = &self.values[start..end];
                if row_slice.iter().any(|v| !v.is_nan() && v.is_finite()) {
                    valid_lines += 1;
                    payload.extend_from_slice(row_slice);
                }
            }
            (payload, profile_length as u32, valid_lines)
        } else {
            let profile_length = self.height;
            let line_count = self.width;
            let mut payload = Vec::with_capacity(profile_length * line_count);
            let mut valid_lines = 0u32;
            for col in 0..line_count {
                let mut has_valid = false;
                for row in 0..profile_length {
                    let idx = row * self.width + col;
                    if let Some(&v) = self.values.get(idx)
                        && !v.is_nan()
                        && v.is_finite()
                    {
                        has_valid = true;
                        break;
                    }
                }
                if has_valid {
                    valid_lines += 1;
                    for row in 0..profile_length {
                        let idx = row * self.width + col;
                        payload.push(self.values.get(idx).copied().unwrap_or(f32::NAN));
                    }
                }
            }
            (payload, profile_length as u32, valid_lines)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MatrixData;

    #[test]
    fn extracts_row_and_column_profiles() {
        let data = MatrixData::new(
            3,
            2,
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            1.0,
            6.0,
            "test".to_string(),
            1,
        );

        assert_eq!(data.extract_1d_line_profile(0, 0), vec![1.0, 2.0, 3.0]);
        assert_eq!(data.extract_1d_line_profile(1, 1), vec![2.0, 5.0]);
        assert_eq!(
            data.extract_all_1d_line_profiles(0),
            vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]
        );
        assert_eq!(
            data.extract_all_1d_line_profiles(1),
            vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]]
        );
    }
}
