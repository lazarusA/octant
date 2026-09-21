//! Variable metadata and spatial role resolution.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct VariableInfo {
    pub name: String,
    pub data_type: String,
    pub shape: Vec<u64>,
    pub dimension_names: Vec<String>,
    pub chunk_shape: Vec<u64>,
    pub file_size: u64,
    pub units: Option<String>,
    pub long_name: Option<String>,
    pub time_coverage_start: Option<String>,
    pub time_coverage_end: Option<String>,
    pub temporal_resolution: Option<String>,
    pub attributes: HashMap<String, String>,
}

impl VariableInfo {
    /// Returns the leaf name of the variable (e.g. "u_wind" for "atmosphere/forecast/u_wind").
    pub fn leaf_name(&self) -> &str {
        let clean = self.name.trim_start_matches('/').trim_end_matches('/');
        clean.rsplit('/').next().unwrap_or(clean)
    }

    /// Returns the group prefix path of the variable (e.g. "atmosphere/forecast" or None for root variables).
    pub fn group_path(&self) -> Option<&str> {
        let clean = self.name.trim_start_matches('/').trim_end_matches('/');
        if let Some(pos) = clean.rfind('/') {
            Some(&clean[..pos])
        } else {
            None
        }
    }

    /// Resolves spatial X, Y, and Z dimension indices for this variable using explicit spatial role configs
    /// or fallback dimension name heuristics.
    pub fn resolve_spatial_dim_indices(
        &self,
        dim_configs: &[crate::app::DimConfig],
    ) -> (Option<usize>, Option<usize>, Option<usize>) {
        let explicit_grid = (0..self.dimension_names.len()).find(|&d| {
            dim_configs
                .get(d)
                .is_some_and(|c| c.spatial == crate::app::SpatialRole::Grid)
        });

        if let Some(grid_d) = explicit_grid {
            let explicit_z = (0..self.dimension_names.len()).find(|&d| {
                dim_configs
                    .get(d)
                    .is_some_and(|c| c.spatial == crate::app::SpatialRole::Z)
            });
            return (Some(grid_d), None, explicit_z);
        }

        let explicit_x = (0..self.dimension_names.len())
            .find(|&d| {
                dim_configs
                    .get(d)
                    .is_some_and(|c| c.spatial == crate::app::SpatialRole::X)
            })
            .or_else(|| {
                self.dimension_names
                    .iter()
                    .rposition(|d| crate::data::coordinates::naming::is_spatial_x_name(d))
            });

        let explicit_y = (0..self.dimension_names.len())
            .find(|&d| {
                dim_configs
                    .get(d)
                    .is_some_and(|c| c.spatial == crate::app::SpatialRole::Y)
            })
            .or_else(|| {
                self.dimension_names
                    .iter()
                    .rposition(|d| crate::data::coordinates::naming::is_spatial_y_name(d))
            });

        let explicit_z = (0..self.dimension_names.len())
            .find(|&d| {
                dim_configs
                    .get(d)
                    .is_some_and(|c| c.spatial == crate::app::SpatialRole::Z)
            })
            .or_else(|| {
                self.dimension_names
                    .iter()
                    .rposition(|d| crate::data::coordinates::naming::is_spatial_z_name(d))
            });

        (explicit_x, explicit_y, explicit_z)
    }
}
