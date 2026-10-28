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
    /// With `cell_edges` the bounds are where the first and last cells end, for plots drawing
    /// whole cells edge to edge; otherwise the first and last values.
    pub fn resolve_axis_bounds_and_title(
        &self,
        dim_idx: usize,
        fallback_name: &str,
        fallback_len: usize,
        cell_edges: bool,
    ) -> ((f64, f64), String, Option<String>) {
        let index_bounds = |first: usize, last: usize| match cell_edges {
            true => (first as f64 - 0.5, last as f64 + 0.5),
            false => (first as f64, last as f64),
        };
        // Values need a span; edges have one even for a single index.
        let last = fallback_len.saturating_sub(1);
        let mut bounds = index_bounds(0, if cell_edges { last } else { last.max(1) });
        let mut name = fallback_name;
        let mut units = None;
        // An OME-Zarr axis unit, shown in the title ("y [µm]").
        let mut axis_unit = None;
        if let Some((meta, var, size, range)) = self.plotted_window(dim_idx) {
            bounds = index_bounds(range.0, range.1);
            if let Some(dim) = var.dimension_names.get(dim_idx) {
                name = dim;
                axis_unit = var.dimension_unit(dim);
                units = dimension_units(meta, var, dim, axis_unit);
                let values =
                    || meta.get_coord_bounds_for_var_range(Some(&var.name), dim, size, range);
                let coords = match cell_edges {
                    // A dimension of one index has no spacing: one unit around its value.
                    true => meta
                        .get_cell_edges_for_var_range(Some(&var.name), dim, size, range)
                        .or_else(|| values().map(|(lo, hi)| (lo - 0.5, hi + 0.5))),
                    false => values(),
                };
                bounds = coords.unwrap_or(bounds);
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

    /// The plotted variable along dimension `dim_idx`: its metadata, the variable, the
    /// dimension's size and its selected index range (the whole dimension when unset).
    pub(crate) fn plotted_window(
        &self,
        dim_idx: usize,
    ) -> Option<(&DatasetMetadata, &VariableInfo, usize, (usize, usize))> {
        let plotted = self.plotted();
        let meta = plotted.metadata.as_ref()?;
        let var = meta.variables.get(plotted.variable_idx)?;
        let size = *var.shape.get(dim_idx)? as usize;
        let range =
            (plotted.dim_ranges.get(dim_idx).copied()).unwrap_or((0, size.saturating_sub(1)));
        Some((meta, var, size, range))
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

#[cfg(test)]
mod tests {
    use crate::app::test_support::{memory_app, selection_of};
    use crate::data::CoordValues;

    #[test]
    fn a_one_index_dimension_keeps_its_coordinate() {
        let (mut app, mut meta) = memory_app();
        // `elev(lat, lon)`'s lat window is one index; its coordinate stands alone.
        let one = CoordValues::Regular {
            start: 45.0,
            step: 0.0,
            len: 1,
        };
        meta.dimension_coordinates.insert("lat".into(), one);
        let mut selection = selection_of(&mut app, &meta, "elev");
        let var = &mut selection.metadata.as_mut().expect("metadata").variables;
        let idx = selection.variable_idx;
        var[idx].shape[0] = 1;
        selection.dim_ranges[0] = (0, 0);
        *app.layers.base.selection_mut() = selection;
        let (bounds, _, _) = app.resolve_axis_bounds_and_title(0, "Y", 1, true);
        assert_eq!(bounds, (44.5, 45.5));
        // Without metadata, one index spans one cell.
        let (empty, _) = memory_app();
        let (bounds, _, _) = empty.resolve_axis_bounds_and_title(0, "Y", 1, true);
        assert_eq!(bounds, (-0.5, 0.5));
    }
}
