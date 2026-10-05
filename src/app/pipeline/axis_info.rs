//! Dimension coordinate bounds and title resolution for plot axes.

use crate::app::OctantApp;

impl OctantApp {
    /// Resolves coordinate bounds, formatted title, and units/time context for a given dimension index.
    pub fn resolve_axis_bounds_and_title(
        &self,
        dim_idx: usize,
        fallback_name: &str,
        fallback_len: usize,
    ) -> ((f64, f64), String, Option<String>) {
        let mut bounds = (0.0, fallback_len.saturating_sub(1).max(1) as f64);
        let mut name = fallback_name.to_string();
        let mut units = None;

        if let Some(meta) = &self.plotted_dataset_metadata
            && let Some(var) = meta.variables.get(self.plotted_variable_idx)
            && dim_idx < var.shape.len()
        {
            let dim_size = var
                .shape
                .get(dim_idx)
                .copied()
                .unwrap_or(fallback_len as u64) as usize;
            let (start_p, end_p) = self
                .plotted_selected_dim_ranges
                .get(dim_idx)
                .copied()
                .unwrap_or((0, dim_size.saturating_sub(1)));
            bounds = (start_p as f64, end_p as f64);

            if let Some(dim_n) = var.dimension_names.get(dim_idx) {
                name = dim_n.clone();
                if let Some(coord_var) = meta.variables.iter().find(|v| {
                    v.name.eq_ignore_ascii_case(dim_n)
                        || v.name
                            .trim_start_matches('/')
                            .eq_ignore_ascii_case(dim_n.trim_start_matches('/'))
                }) {
                    units = coord_var
                        .units
                        .clone()
                        .or_else(|| coord_var.attributes.get("units").cloned());
                    if units.is_none() {
                        units = coord_var.attributes.get("time_coverage_start").cloned();
                    }
                }
                if units.is_none() {
                    units = var
                        .units
                        .clone()
                        .or_else(|| var.attributes.get("units").cloned())
                        .or_else(|| var.attributes.get("time_coverage_start").cloned());
                }
                if let Some(coord_bounds) = meta.get_coord_bounds_for_var_range(
                    Some(&var.name),
                    dim_n,
                    dim_size,
                    (start_p, end_p),
                ) {
                    bounds = coord_bounds;
                }
            }
        }

        let title =
            crate::data::coordinates::naming::format_dimension_axis_title(&name).into_owned();
        (bounds, title, units)
    }

    /// Returns a human-friendly label for the spatial axis (e.g., "Along X (lon)", "Along Z (depth)").
    pub fn get_spatial_dim_label(&self, axis: usize) -> String {
        let (name_opt, fallback) = match axis {
            0 => (self.get_spatial_dim_name(0), "X / Longitude"),
            1 => (self.get_spatial_dim_name(1), "Y / Latitude"),
            _ => (self.get_spatial_dim_name(2), "Z / Depth"),
        };

        if let Some(name) = name_opt {
            let axis_letter = match axis {
                0 => "X",
                1 => "Y",
                _ => "Z",
            };
            format!("Along {} ({})", axis_letter, name)
        } else {
            format!("Along {}", fallback)
        }
    }
}
