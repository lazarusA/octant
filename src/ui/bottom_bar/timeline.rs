//! Labels of the animated dimension (current, first and last step, step
//! size), computed once per frame for measuring and drawing the bottom bar.

use crate::app::OctantApp;

/// Owned per-frame snapshot, so the bar can mutate `app` while drawing.
/// Only these four labels are allocated; metadata is borrowed while building.
pub(super) struct Timeline {
    pub current: String,
    pub start: String,
    pub end: String,
    pub step_size: String,
    pub step: usize,
    pub last_step: usize,
}

impl Timeline {
    pub fn from_app(app: &OctantApp) -> Self {
        let max_steps = app.animated_dim_extent();
        let last_step = max_steps.saturating_sub(1);
        let meta = app.plotted_dataset_metadata.as_ref();
        let var = meta.and_then(|m| m.variables.get(app.plotted_variable_idx));
        let dim = dim_name(app);
        let coords = meta.and_then(|m| m.get_dim_coords(var.map(|v| v.name.as_str()), dim));

        let label = |step: usize, coord: Option<&String>| match coord
            .map(String::as_str)
            .filter(|c| is_display_coord(c))
        {
            Some(coord) => coord.to_owned(),
            None => crate::utils::units::format_axis_value(
                step,
                max_steps,
                var.map(|_| dim),
                var.and_then(|v| v.units.as_deref()),
                var.and_then(|v| v.time_coverage_start.as_deref()),
                var.and_then(|v| v.temporal_resolution.as_deref()),
                Some(&app.plotted_store_target_input),
            ),
        };

        let step = app.current_timestep;
        Self {
            current: label(step, coords.and_then(|c| c.get(step))),
            start: label(0, coords.and_then(|c| c.first())),
            end: label(last_step, coords.and_then(|c| c.last())),
            step_size: var
                .and_then(|v| v.temporal_resolution.clone())
                .unwrap_or_else(|| "Step: 1".to_owned()),
            step,
            last_step,
        }
    }
}

/// Name of the plotted animated dimension, or `"step"` without metadata.
pub(super) fn dim_name(app: &OctantApp) -> &str {
    app.plotted_dataset_metadata
        .as_ref()
        .and_then(|m| m.variables.get(app.plotted_variable_idx))
        .and_then(|v| v.dimension_names.get(app.plotted_animated_dim.unwrap_or(0)))
        .map_or("step", String::as_str)
}

/// Whether a coordinate value reads well as-is. Bare numbers (e.g. `42.5`)
/// are replaced by a formatted axis value; dates, times and paths are kept.
pub(super) fn is_display_coord(coord: &str) -> bool {
    let raw_number = coord.parse::<f64>().is_ok() && !coord.contains(['-', ':', '/', 'T']);
    !raw_number && !coord.trim().is_empty()
}
