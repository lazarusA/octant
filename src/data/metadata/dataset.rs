//! Dataset metadata and coordinate boundary queries.

use std::collections::HashMap;

use super::{coord_values::CoordValues, tree::VariableTreeGroup, variable::VariableInfo};

#[derive(Debug, Clone, Default)]
pub struct DatasetMetadata {
    pub name: String,
    pub store_type: String,
    pub variables: Vec<VariableInfo>,
    pub dimension_coordinates: HashMap<String, CoordValues>,
    /// What an OME-Zarr store holds beyond its variables (plates, series names).
    pub ome: Option<std::sync::Arc<crate::data::ome::OmeDataset>>,
}

impl DatasetMetadata {
    /// Look up the dimension coordinates for a given variable and dimension name,
    /// checking scoped `{var_name}/{dim_name}` first before falling back to `{dim_name}`.
    /// Keys are lowercased into a stack buffer, so per-frame lookups do not allocate.
    pub fn get_dim_coords(&self, var_name: Option<&str>, dim_name: &str) -> Option<&CoordValues> {
        let clean = dim_name.trim();
        let mut buf = [0u8; KEY_BUF];
        (var_name.and_then(|v| self.coords_for(&mut buf, &[v.trim(), "/", clean])))
            .or_else(|| self.coords_for(&mut buf, &[clean]))
            .or_else(|| self.dimension_coordinates.get(dim_name))
    }

    /// The coordinates under `parts` joined and lowercased: ASCII in `buf`, other text
    /// by Unicode rules on the heap.
    fn coords_for(&self, buf: &mut [u8; KEY_BUF], parts: &[&str]) -> Option<&CoordValues> {
        match lower_key(buf, parts) {
            Some(key) => self.dimension_coordinates.get(key),
            None => self
                .dimension_coordinates
                .get(&parts.concat().to_lowercase()),
        }
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
        let dim_len = coords.len().max(dim_size).max(1);
        coords.range_bounds(range.0.min(range.1), range.0.max(range.1), dim_len)
    }

    /// The outer cell edges of indices `range` of `dim_name` of `var_name`
    /// ([`CoordValues::cell_edges`]): where the cells drawn for those indices end.
    pub fn get_cell_edges_for_var_range(
        &self,
        var_name: Option<&str>,
        dim_name: &str,
        dim_size: usize,
        range: (usize, usize),
    ) -> Option<(f64, f64)> {
        let coords = self.get_dim_coords(var_name, dim_name)?;
        let dim_len = coords.len().max(dim_size).max(1);
        coords.cell_edges(range.0, range.1, dim_len)
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
        let mut tree = VariableTreeGroup::build_tree_from_variables(&self.variables);
        if let Some(ome) = &self.ome
            && !ome.group_names.is_empty()
        {
            tree.apply_labels(&|path| ome.group_name(path).map(str::to_string));
        }
        tree
    }

    /// Builds a hierarchical variable tree from a slice of VariableInfo.
    pub fn build_tree_from_variables(variables: &[VariableInfo]) -> VariableTreeGroup {
        VariableTreeGroup::build_tree_from_variables(variables)
    }
}

/// Longest coordinate key looked up without allocating.
const KEY_BUF: usize = 128;

/// `parts` joined and ASCII-lowercased in `buf`; `None` when they hold non-ASCII text
/// (lowercased by Unicode rules instead) or do not fit.
fn lower_key<'b>(buf: &'b mut [u8; KEY_BUF], parts: &[&str]) -> Option<&'b str> {
    let mut len = 0;
    for part in parts {
        let end = len + part.len();
        if !part.is_ascii() || end > KEY_BUF {
            return None;
        }
        buf[len..end].copy_from_slice(part.as_bytes());
        len = end;
    }
    buf[..len].make_ascii_lowercase();
    std::str::from_utf8(&buf[..len]).ok()
}
