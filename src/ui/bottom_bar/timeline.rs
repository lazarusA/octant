//! Labels of the animated dimension (current, first and last step, step
//! size) for the bottom bar, cached until the step or variable changes.

use crate::app::OctantApp;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Owned snapshot of the timeline labels, so the bar can mutate `app` while
/// drawing. Built only when its [`TimelineKey`] changes; otherwise reused.
#[derive(Clone, Debug)]
pub(super) struct Timeline {
    pub current: String,
    pub start: String,
    pub end: String,
    pub step_size: String,
    pub step: usize,
    pub last_step: usize,
}

/// Everything the labels depend on. Hashing the store target and the
/// metadata address detects a newly loaded dataset without allocating.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TimelineKey {
    variable: usize,
    animated_dim: Option<usize>,
    step: usize,
    extent: usize,
    dataset: u64,
}

impl TimelineKey {
    fn from_app(app: &OctantApp) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        app.plotted_store_target_input.hash(&mut hasher);
        std::ptr::hash(
            app.plotted_dataset_metadata
                .as_ref()
                .map_or(std::ptr::null(), |m| m as *const _),
            &mut hasher,
        );
        Self {
            variable: app.plotted_variable_idx,
            animated_dim: app.plotted_animated_dim,
            step: app.current_timestep,
            extent: app.animated_dim_extent(),
            dataset: hasher.finish(),
        }
    }
}

impl Timeline {
    /// Cached labels for the current frame. A cache hit only bumps an `Arc`
    /// refcount; labels are rebuilt when the key changes (e.g. each new step).
    pub fn cached(app: &OctantApp, ctx: &egui::Context) -> Arc<Self> {
        let id = egui::Id::new(("bottom_bar", "timeline"));
        let key = TimelineKey::from_app(app);
        if let Some((cached_key, timeline)) =
            ctx.data(|d| d.get_temp::<(TimelineKey, Arc<Self>)>(id))
            && cached_key == key
        {
            return timeline;
        }
        let timeline = Arc::new(Self::from_app(app));
        ctx.data_mut(|d| d.insert_temp(id, (key, Arc::clone(&timeline))));
        timeline
    }

    fn from_app(app: &OctantApp) -> Self {
        let max_steps = app.animated_dim_extent();
        let last_step = max_steps.saturating_sub(1);
        let meta = app.plotted_dataset_metadata.as_ref();
        let var = meta.and_then(|m| m.variables.get(app.plotted_variable_idx));
        let dim = dim_name(app);
        let coords = meta.and_then(|m| m.get_dim_coords(var.map(|v| v.name.as_str()), dim));
        let coord =
            |pick: fn(&[String]) -> Option<&String>| coords.and_then(pick).map(String::as_str);

        let label = |step: usize, coord: Option<&str>| match coord.filter(|c| is_display_coord(c)) {
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
            current: label(step, coords.and_then(|c| c.get(step)).map(String::as_str)),
            start: label(0, coord(<[String]>::first)),
            end: label(last_step, coord(<[String]>::last)),
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

/// Whether a coordinate value reads well as-is. Anything that parses as a
/// number (`42.5`, `-90`, `1e-3`) is replaced by a formatted axis value;
/// dates, times and other text are kept.
pub(super) fn is_display_coord(coord: &str) -> bool {
    let coord = coord.trim();
    !coord.is_empty() && coord.parse::<f64>().is_err()
}
