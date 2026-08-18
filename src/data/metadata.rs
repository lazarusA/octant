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

    /// Returns numerical coordinate bounds for a subrange `(start_idx, end_idx)` within `dim_size` for `dim_name`.
    pub fn get_coord_bounds_for_range(
        &self,
        dim_name: &str,
        dim_size: usize,
        range: (usize, usize),
    ) -> Option<(f64, f64)> {
        let clean = dim_name.trim().to_lowercase();
        let coords = self
            .dimension_coordinates
            .get(&clean)
            .or_else(|| self.dimension_coordinates.get(dim_name))?;

        let (start, end) = (range.0.min(range.1), range.0.max(range.1));
        let total_len = dim_size.max(coords.len()).max(1);

        // If the full coordinate vector is available with individual coordinate values
        if coords.len() >= total_len && coords.len() > end {
            let start_val: f64 = coords.get(start)?.parse().ok()?;
            let end_val: f64 = coords.get(end)?.parse().ok()?;
            return Some((start_val.min(end_val), start_val.max(end_val)));
        }

        // If only boundary coordinates (first, last) are available
        let first: f64 = coords.first()?.parse().ok()?;
        let last: f64 = coords.last()?.parse().ok()?;
        if total_len <= 1 {
            return Some((first.min(last), first.max(last)));
        }

        let t_start = start as f64 / (total_len - 1) as f64;
        let t_end = end as f64 / (total_len - 1) as f64;

        let val_start = first + t_start * (last - first);
        let val_end = first + t_end * (last - first);

        Some((val_start.min(val_end), val_start.max(val_end)))
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
    /// Resolves spatial X, Y, and Z dimension indices for this variable using explicit spatial role configs
    /// or fallback dimension name heuristics.
    pub fn resolve_spatial_dim_indices(
        &self,
        dim_configs: &[crate::app::DimConfig],
    ) -> (Option<usize>, Option<usize>, Option<usize>) {
        let explicit_x = (0..self.dimension_names.len())
            .find(|&d| {
                dim_configs
                    .get(d)
                    .is_some_and(|c| c.spatial == crate::app::SpatialRole::X)
            })
            .or_else(|| {
                self.dimension_names
                    .iter()
                    .rposition(|d| crate::utils::coordinates::is_spatial_x_name(d))
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
                    .rposition(|d| crate::utils::coordinates::is_spatial_y_name(d))
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
                    .rposition(|d| crate::utils::coordinates::is_spatial_z_name(d))
            });

        (explicit_x, explicit_y, explicit_z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coord_bounds_for_range_with_boundary_coords() {
        let mut meta = DatasetMetadata::default();
        meta.dimension_coordinates.insert(
            "lat".to_string(),
            vec!["-90.0".to_string(), "90.0".to_string()],
        );

        // 101 points from index 0 (-90) to index 100 (+90)
        let bounds_full = meta.get_coord_bounds_for_range("lat", 101, (0, 100));
        assert_eq!(bounds_full, Some((-90.0, 90.0)));

        let bounds_sub = meta.get_coord_bounds_for_range("lat", 101, (25, 75));
        assert_eq!(bounds_sub, Some((-45.0, 45.0)));

        let bounds_single = meta.get_coord_bounds_for_range("lat", 101, (50, 50));
        assert_eq!(bounds_single, Some((0.0, 0.0)));
    }

    #[test]
    fn test_coord_bounds_for_range_with_descending_boundary_coords() {
        let mut meta = DatasetMetadata::default();
        meta.dimension_coordinates.insert(
            "lat".to_string(),
            vec!["90.0".to_string(), "-90.0".to_string()],
        );

        // 101 points: index 0 is +90 (North), index 100 is -90 (South)
        // Range (0, 50) is Northern hemisphere [0..90]
        let bounds_north = meta.get_coord_bounds_for_range("lat", 101, (0, 50));
        assert_eq!(bounds_north, Some((0.0, 90.0)));

        // Range (50, 100) is Southern hemisphere [-90..0]
        let bounds_south = meta.get_coord_bounds_for_range("lat", 101, (50, 100));
        assert_eq!(bounds_south, Some((-90.0, 0.0)));
    }

    #[test]
    fn test_coord_bounds_for_range_with_full_coords() {
        let mut meta = DatasetMetadata::default();
        meta.dimension_coordinates.insert(
            "lon".to_string(),
            vec![
                "0.0".to_string(),
                "10.0".to_string(),
                "25.0".to_string(),
                "50.0".to_string(),
            ],
        );

        let bounds_sub = meta.get_coord_bounds_for_range("lon", 4, (1, 3));
        assert_eq!(bounds_sub, Some((10.0, 50.0)));
    }
}
