//! Metadata representations for open dataset sources and variables.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct DatasetMetadata {
    pub name: String,
    pub store_type: String,
    pub variables: Vec<VariableInfo>,
    pub dimension_coordinates: HashMap<String, Vec<String>>,
}

impl DatasetMetadata {
    /// Returns numerical min/max coordinate bounds for a dimension name if available.
    pub fn get_coord_bounds(&self, dim_name: &str) -> Option<(f64, f64)> {
        let clean = dim_name.trim().to_lowercase();
        let coords = self
            .dimension_coordinates
            .get(&clean)
            .or_else(|| self.dimension_coordinates.get(dim_name))?;
        let (first, last) = (coords.first()?, coords.last()?);
        let f_v: f64 = first.parse().ok()?;
        let l_v: f64 = last.parse().ok()?;
        Some((f_v.min(l_v), f_v.max(l_v)))
    }
}

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
    /// Resolves spatial X and Y dimension indices for this variable using explicit spatial role configs
    /// or fallback dimension name heuristics.
    pub fn resolve_spatial_dim_indices(
        &self,
        dim_configs: &[crate::app::DimensionConfig],
    ) -> (Option<usize>, Option<usize>) {
        let explicit_x = (0..self.dimension_names.len()).find(|&d| {
            dim_configs
                .get(d)
                .is_some_and(|c| c.spatial == crate::app::SpatialRole::X)
        }).or_else(|| {
            self.dimension_names.iter().rposition(|d| {
                let clean = d.to_lowercase();
                clean.contains("lon") || clean == "x" || clean.contains("col")
            })
        });

        let explicit_y = (0..self.dimension_names.len()).find(|&d| {
            dim_configs
                .get(d)
                .is_some_and(|c| c.spatial == crate::app::SpatialRole::Y)
        }).or_else(|| {
            self.dimension_names.iter().rposition(|d| {
                let clean = d.to_lowercase();
                clean.contains("lat") || clean == "y" || clean.contains("row")
            })
        });

        (explicit_x, explicit_y)
    }
}
