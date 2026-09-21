//! Dataset metadata and coordinate boundary queries.

use std::collections::HashMap;

use super::{tree::VariableTreeGroup, variable::VariableInfo};

#[derive(Debug, Clone, Default)]
pub struct DatasetMetadata {
    pub name: String,
    pub store_type: String,
    pub variables: Vec<VariableInfo>,
    pub dimension_coordinates: HashMap<String, Vec<String>>,
}

impl DatasetMetadata {
    /// Look up the dimension coordinate slice for a given variable and dimension name,
    /// checking scoped `{var_name}/{dim_name}` first before falling back to `{dim_name}`.
    pub fn get_dim_coords(&self, var_name: Option<&str>, dim_name: &str) -> Option<&[String]> {
        let clean = dim_name.trim().to_lowercase();
        if let Some(v) = var_name {
            let scoped = format!("{}/{}", v.trim().to_lowercase(), clean);
            if let Some(coords) = self.dimension_coordinates.get(&scoped) {
                return Some(coords.as_slice());
            }
        }
        self.dimension_coordinates
            .get(&clean)
            .or_else(|| self.dimension_coordinates.get(dim_name))
            .map(|c| c.as_slice())
    }

    /// Returns numerical min/max coordinate bounds for a variable dimension name if available.
    pub fn get_coord_bounds_for_var(
        &self,
        var_name: Option<&str>,
        dim_name: &str,
    ) -> Option<(f64, f64)> {
        let coords = self.get_dim_coords(var_name, dim_name)?;
        let (first, last) = (coords.first()?, coords.last()?);
        let f_v: f64 = first.parse().ok()?;
        let l_v: f64 = last.parse().ok()?;
        Some((f_v.min(l_v), f_v.max(l_v)))
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
