//! Dataset metadata and coordinate boundary queries.

use std::collections::HashMap;

use super::{coord_values::CoordValues, tree::VariableTreeGroup, variable::VariableInfo};

#[derive(Debug, Clone, Default)]
pub struct DatasetMetadata {
    pub name: String,
    pub store_type: String,
    pub variables: Vec<VariableInfo>,
    pub dimension_coordinates: HashMap<String, CoordValues>,
}

impl DatasetMetadata {
    /// Look up the dimension coordinates for a given variable and dimension name,
    /// checking scoped `{var_name}/{dim_name}` first before falling back to `{dim_name}`.
    pub fn get_dim_coords(&self, var_name: Option<&str>, dim_name: &str) -> Option<&CoordValues> {
        let clean = dim_name.trim().to_lowercase();
        if let Some(v) = var_name {
            let scoped = format!("{}/{}", v.trim().to_lowercase(), clean);
            if let Some(coords) = self.dimension_coordinates.get(&scoped) {
                return Some(coords);
            }
        }
        self.dimension_coordinates
            .get(&clean)
            .or_else(|| self.dimension_coordinates.get(dim_name))
    }

    /// Returns numerical min/max coordinate bounds for a variable dimension name if available.
    pub fn get_coord_bounds_for_var(
        &self,
        var_name: Option<&str>,
        dim_name: &str,
    ) -> Option<(f64, f64)> {
        let coords = self.get_dim_coords(var_name, dim_name)?;
        let last = coords.len().checked_sub(1)?;
        coords.range_bounds(0, last, coords.len())
    }

    /// Returns numerical min/max coordinate bounds for a dimension name if available.
    pub fn get_coord_bounds(&self, dim_name: &str) -> Option<(f64, f64)> {
        self.get_coord_bounds_for_var(None, dim_name)
    }

    /// Returns numerical coordinate bounds for a subrange `(start_idx, end_idx)` within `dim_size` for `dim_name` of `var_name`.
    pub fn get_coord_bounds_for_var_range(
        &self,
        var_name: Option<&str>,
        dim_name: &str,
        dim_size: usize,
        range: (usize, usize),
    ) -> Option<(f64, f64)> {
        let coords = self.get_dim_coords(var_name, dim_name)?;
        let dim_len = if coords.len() > dim_size {
            coords.len()
        } else {
            dim_size.max(1)
        };
        coords.range_bounds(range.0.min(range.1), range.0.max(range.1), dim_len)
    }

    /// Returns numerical coordinate bounds for a subrange `(start_idx, end_idx)` within `dim_size` for `dim_name`.
    pub fn get_coord_bounds_for_range(
        &self,
        dim_name: &str,
        dim_size: usize,
        range: (usize, usize),
    ) -> Option<(f64, f64)> {
        self.get_coord_bounds_for_var_range(None, dim_name, dim_size, range)
    }

    /// Builds a hierarchical variable tree from the flat `variables` list.
    pub fn build_variable_tree(&self) -> VariableTreeGroup {
        VariableTreeGroup::build_tree_from_variables(&self.variables)
    }

    /// Builds a hierarchical variable tree from a slice of VariableInfo.
    pub fn build_tree_from_variables(variables: &[VariableInfo]) -> VariableTreeGroup {
        VariableTreeGroup::build_tree_from_variables(variables)
    }
}
