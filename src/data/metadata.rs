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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VariableTreeGroup {
    /// Segment name of this group (e.g. "atmosphere", "forecast", or "Root").
    pub name: String,
    /// Full path of this group (e.g. "atmosphere/forecast" or "" for root).
    pub full_path: String,
    /// Indices of variables directly belonging to this group (into `DatasetMetadata::variables`).
    pub variable_indices: Vec<usize>,
    /// Child subgroups.
    pub subgroups: Vec<VariableTreeGroup>,
}

impl VariableTreeGroup {
    /// Returns the total number of variables in this group and all its descendant subgroups.
    pub fn total_variable_count(&self) -> usize {
        let direct = self.variable_indices.len();
        let nested: usize = self
            .subgroups
            .iter()
            .map(|g| g.total_variable_count())
            .sum();
        direct + nested
    }

    /// Recursively filters the tree according to a search query string.
    /// Returns `Some(filtered_group)` if this group, any of its subgroups, or any of its variables match.
    pub fn filter(&self, query: &str, variables: &[VariableInfo]) -> Option<VariableTreeGroup> {
        let query_lower = query.trim().to_lowercase();
        if query_lower.is_empty() {
            return Some(self.clone());
        }
        self.filter_lowercased(&query_lower, variables)
    }

    fn filter_lowercased(
        &self,
        query_lower: &str,
        variables: &[VariableInfo],
    ) -> Option<VariableTreeGroup> {
        let group_matches = self.name.to_lowercase().contains(query_lower)
            || self.full_path.to_lowercase().contains(query_lower);

        let mut filtered_vars = Vec::new();
        for &idx in &self.variable_indices {
            if let Some(var) = variables.get(idx)
                && (group_matches
                    || var.name.to_lowercase().contains(query_lower)
                    || var
                        .long_name
                        .as_deref()
                        .is_some_and(|l| l.to_lowercase().contains(query_lower))
                    || var
                        .units
                        .as_deref()
                        .is_some_and(|u| u.to_lowercase().contains(query_lower)))
            {
                filtered_vars.push(idx);
            }
        }

        let mut filtered_subgroups = Vec::new();
        for sub in &self.subgroups {
            if let Some(filtered_sub) = sub.filter_lowercased(query_lower, variables) {
                filtered_subgroups.push(filtered_sub);
            }
        }

        if !filtered_vars.is_empty() || !filtered_subgroups.is_empty() {
            Some(VariableTreeGroup {
                name: self.name.clone(),
                full_path: self.full_path.clone(),
                variable_indices: filtered_vars,
                subgroups: filtered_subgroups,
            })
        } else {
            None
        }
    }
}

impl DatasetMetadata {
    /// Builds a hierarchical variable tree from the flat `variables` list.
    /// Handles root variables, arbitrary nesting depths, and mixed structures cleanly.
    pub fn build_variable_tree(&self) -> VariableTreeGroup {
        Self::build_tree_from_variables(&self.variables)
    }

    /// Builds a hierarchical variable tree from a slice of VariableInfo.
    pub fn build_tree_from_variables(variables: &[VariableInfo]) -> VariableTreeGroup {
        let mut root = VariableTreeGroup {
            name: "Root".to_string(),
            full_path: String::new(),
            variable_indices: Vec::new(),
            subgroups: Vec::new(),
        };

        for (idx, var) in variables.iter().enumerate() {
            let clean_name = var.name.trim_start_matches('/').trim_end_matches('/');
            let segments: Vec<&str> = clean_name.split('/').filter(|s| !s.is_empty()).collect();

            if segments.len() <= 1 {
                // Root-level variable
                root.variable_indices.push(idx);
            } else {
                // Nested variable: traverse / insert subgroups
                let mut current_group = &mut root;
                let mut current_path = String::new();

                for &seg in &segments[..segments.len() - 1] {
                    if !current_path.is_empty() {
                        current_path.push('/');
                    }
                    current_path.push_str(seg);

                    let pos = current_group.subgroups.iter().position(|g| g.name == seg);

                    let sub_idx = match pos {
                        Some(p) => p,
                        None => {
                            current_group.subgroups.push(VariableTreeGroup {
                                name: seg.to_string(),
                                full_path: current_path.clone(),
                                variable_indices: Vec::new(),
                                subgroups: Vec::new(),
                            });
                            current_group.subgroups.len() - 1
                        }
                    };

                    current_group = &mut current_group.subgroups[sub_idx];
                }

                current_group.variable_indices.push(idx);
            }
        }

        root
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

    #[test]
    fn test_variable_tree_mixed_root_and_nested() {
        let vars = vec![
            VariableInfo {
                name: "elevation".to_string(),
                ..Default::default()
            },
            VariableInfo {
                name: "mask".to_string(),
                ..Default::default()
            },
            VariableInfo {
                name: "atmosphere/surface_pressure".to_string(),
                ..Default::default()
            },
            VariableInfo {
                name: "atmosphere/forecast/u_wind".to_string(),
                ..Default::default()
            },
            VariableInfo {
                name: "atmosphere/forecast/v_wind".to_string(),
                ..Default::default()
            },
            VariableInfo {
                name: "ocean/temperature".to_string(),
                ..Default::default()
            },
        ];

        let tree = DatasetMetadata::build_tree_from_variables(&vars);

        assert_eq!(tree.variable_indices, vec![0, 1]); // elevation, mask
        assert_eq!(tree.subgroups.len(), 2); // atmosphere, ocean
        assert_eq!(tree.total_variable_count(), 6);

        let atmo = &tree.subgroups[0];
        assert_eq!(atmo.name, "atmosphere");
        assert_eq!(atmo.full_path, "atmosphere");
        assert_eq!(atmo.variable_indices, vec![2]); // surface_pressure
        assert_eq!(atmo.subgroups.len(), 1); // forecast

        let forecast = &atmo.subgroups[0];
        assert_eq!(forecast.name, "forecast");
        assert_eq!(forecast.full_path, "atmosphere/forecast");
        assert_eq!(forecast.variable_indices, vec![3, 4]); // u_wind, v_wind
        assert_eq!(forecast.total_variable_count(), 2);

        let ocean = &tree.subgroups[1];
        assert_eq!(ocean.name, "ocean");
        assert_eq!(ocean.full_path, "ocean");
        assert_eq!(ocean.variable_indices, vec![5]); // temperature
    }

    #[test]
    fn test_variable_tree_filter() {
        let vars = vec![
            VariableInfo {
                name: "elevation".to_string(),
                ..Default::default()
            },
            VariableInfo {
                name: "atmosphere/forecast/u_wind".to_string(),
                ..Default::default()
            },
            VariableInfo {
                name: "ocean/temperature".to_string(),
                ..Default::default()
            },
        ];

        let tree = DatasetMetadata::build_tree_from_variables(&vars);

        // Search for "wind"
        let filtered_wind = tree.filter("wind", &vars).expect("should match wind");
        assert_eq!(filtered_wind.variable_indices.len(), 0);
        assert_eq!(filtered_wind.subgroups.len(), 1);
        assert_eq!(filtered_wind.subgroups[0].name, "atmosphere");
        assert_eq!(filtered_wind.subgroups[0].subgroups[0].name, "forecast");
        assert_eq!(
            filtered_wind.subgroups[0].subgroups[0].variable_indices,
            vec![1]
        );

        // Search for "ocean"
        let filtered_ocean = tree.filter("ocean", &vars).expect("should match ocean");
        assert_eq!(filtered_ocean.subgroups.len(), 1);
        assert_eq!(filtered_ocean.subgroups[0].name, "ocean");
        assert_eq!(filtered_ocean.subgroups[0].variable_indices, vec![2]);

        // Non-existent search
        assert!(tree.filter("nonexistent", &vars).is_none());
    }
}
