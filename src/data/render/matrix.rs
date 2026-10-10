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

    /// The categories found by `compute_unique_values` when the matrix was
    /// built. A plain field read, so per-frame code (colorbars, color
    /// params) may call it every frame: nothing is scanned or allocated.
    pub fn unique_values(&self) -> Option<&[f32]> {
        self.unique_values.as_deref()
    }
}
