//! Dimension coordinate bounds and title resolution for plot axes.

use crate::app::OctantApp;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::i18n::{HoverKey, filled, tr};

/// The units of dimension `dim` of `var`: its coordinate variable's units
/// (or `time_coverage_start`), else the OME-Zarr `axis_unit`, else the
/// variable's own.
fn dimension_units(
    meta: &DatasetMetadata,
    var: &VariableInfo,
    dim: &str,
    axis_unit: Option<&str>,
) -> Option<String> {
    let attr = |v: &VariableInfo, key: &str| v.attributes.get(key).cloned();
    let coord_var = meta.variables.iter().find(|v| {
        v.name.eq_ignore_ascii_case(dim)
            || v.name
                .trim_start_matches('/')
                .eq_ignore_ascii_case(dim.trim_start_matches('/'))
    });
    coord_var
        .and_then(|c| {
            (c.units.clone())
                .or_else(|| attr(c, "units"))
                .or_else(|| attr(c, "time_coverage_start"))
        })
        .or_else(|| axis_unit.map(str::to_owned))
        .or_else(|| var.units.clone())
        .or_else(|| attr(var, "units"))
        .or_else(|| attr(var, "time_coverage_start"))
}

impl OctantApp {
    /// Resolves coordinate bounds, formatted title, and units/time context for a given dimension index.
    pub fn resolve_axis_bounds_and_title(
        &self,
        dim_idx: usize,
        fallback_name: &str,
        fallback_len: usize,
    ) -> ((f64, f64), String, Option<String>) {
        let mut bounds = (0.0, fallback_len.saturating_sub(1).max(1) as f64);
        let mut name = fallback_name;
        let mut units = None;
        // An OME-Zarr axis unit, shown in the title ("y [µm]").
        let mut axis_unit = None;
        let plotted = self.plotted();
        if let Some(meta) = &plotted.metadata
            && let Some(var) = meta.variables.get(plotted.variable_idx)
            && let Some(&size) = var.shape.get(dim_idx)
        {
            let size = size as usize;
            let range =
                (plotted.dim_ranges.get(dim_idx).copied()).unwrap_or((0, size.saturating_sub(1)));
            bounds = (range.0 as f64, range.1 as f64);
            if let Some(dim) = var.dimension_names.get(dim_idx) {
                name = dim;
                axis_unit = var.dimension_unit(dim);
                units = dimension_units(meta, var, dim, axis_unit);
                bounds = meta
                    .get_coord_bounds_for_var_range(Some(&var.name), dim, size, range)
                    .unwrap_or(bounds);
            }
        }
        let title = match axis_unit {
            Some(unit) => format!("{name} [{unit}]"),
            None => {
                crate::data::coordinates::naming::format_dimension_axis_title(name).into_owned()
            }
        };
        (bounds, title, units)
    }

    /// Returns a human-friendly label for the spatial axis (e.g., "Along X (lon)", "Along Z (depth)").
    pub fn get_spatial_dim_label(&self, axis: usize) -> String {
        let (name_opt, fallback) = match axis {
            0 => (self.get_spatial_dim_name(0), HoverKey::AxisXLongitude),
            1 => (self.get_spatial_dim_name(1), HoverKey::AxisYLatitude),
            _ => (self.get_spatial_dim_name(2), HoverKey::AxisZDepth),
        };

        if let Some(name) = name_opt {
            let axis_letter = match axis {
                0 => "X",
                1 => "Y",
                _ => "Z",
            };
            filled(tr(HoverKey::AlongAxisNamed), &[&axis_letter, &name])
        } else {
            filled(tr(HoverKey::AlongAxis), &[&tr(fallback)])
        }
    }
}
