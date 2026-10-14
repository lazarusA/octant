//! Labels of the animated dimension (current, first and last step, step
//! size) for the bottom bar, cached until the step or variable changes.

use crate::app::OctantApp;
use crate::i18n::{PlaybackKey, tr};
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

/// Everything the labels depend on. Hashing the store target, the plotted
/// metadata's generation and the coordinates revision detects a newly loaded
/// dataset or newly read coordinates without allocating.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TimelineKey {
    variable: usize,
    animated_dim: Option<usize>,
    step: usize,
    extent: usize,
    dataset: u64,
    /// The labels' language, so switching it rebuilds them.
    language: crate::i18n::Language,
}

impl TimelineKey {
    fn from_app(app: &OctantApp) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        app.plotted().store_target.hash(&mut hasher);
        (app.plotted().metadata_generation, app.coordinates_revision).hash(&mut hasher);
        Self {
            variable: app.plotted().variable_idx,
            animated_dim: app.plotted().animated_dim,
            step: app.playback.current_timestep,
            extent: app.animated_dim_extent(),
            dataset: hasher.finish(),
            language: crate::i18n::language(),
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
        let meta = app.plotted().metadata.as_ref();
        let var = meta.and_then(|m| m.variables.get(app.plotted().variable_idx));
        let dim = dim_name(app);
        // Labels are read by index only when there is one per step.
        let coords = meta
            .and_then(|m| m.get_dim_coords(var.map(|v| v.name.as_str()), dim))
            .filter(|c| c.matches(max_steps));
        let label =
            |i: usize| step_label(app, var, dim, i, max_steps, coords.and_then(|c| c.label(i)));
        let step = app.playback.current_timestep;
        Self {
            current: label(step),
            start: label(0),
            end: label(last_step),
            step_size: var
                .and_then(|v| v.temporal_resolution.clone())
                .or_else(|| ome_step_size(var?, dim))
                .unwrap_or_else(|| tr(PlaybackKey::DefaultStepSize).to_owned()),
            step,
            last_step,
        }
    }
}

/// The label of step `step` of `max_steps` along `dim`: its coordinate
/// `coord` when that reads well as-is, else a formatted axis value (an
/// OME-Zarr time axis with a unit reads its physical time, "12 s").
fn step_label(
    app: &OctantApp,
    var: Option<&crate::data::VariableInfo>,
    dim: &str,
    step: usize,
    max_steps: usize,
    coord: Option<&str>,
) -> String {
    if let Some(coord) = coord.filter(|c| is_display_coord(c)) {
        return coord.to_owned();
    }
    let target = Some(app.plotted().store_target.as_str());
    if var.and_then(|v| v.dimension_unit(dim)).is_some() {
        let meta = app.plotted().metadata.as_ref();
        return crate::ui::hover::format::format_dimension_coord(
            meta, var, target, dim, step, max_steps, None,
        )
        .value;
    }
    crate::utils::units::format_axis_value(
        step,
        max_steps,
        var.map(|_| dim),
        var.and_then(|v| v.units.as_deref()),
        var.and_then(|v| v.time_coverage_start.as_deref()),
        var.and_then(|v| v.temporal_resolution.as_deref()),
        target,
    )
}

/// The time between steps of an OME-Zarr image's animated axis ("2 s").
fn ome_step_size(var: &crate::data::VariableInfo, dim: &str) -> Option<String> {
    let unit = var.dimension_unit(dim)?;
    // Scales are stored as `f32`: shown at that precision (0.1, not
    // 0.10000000149011612).
    let step = var.ome_scale(var.dimension_index(dim)?)? as f32;
    Some(format!(
        "{} {unit}",
        crate::i18n::number::Num(format_args!("{step}"))
    ))
}

/// Name of the plotted animated dimension, or `"step"` without metadata.
pub(super) fn dim_name(app: &OctantApp) -> &str {
    app.plotted()
        .metadata
        .as_ref()
        .and_then(|m| m.variables.get(app.plotted().variable_idx))
        .and_then(|v| {
            v.dimension_names
                .get(app.plotted().animated_dim.unwrap_or(0))
        })
        .map_or("step", String::as_str)
}

/// Whether a coordinate value reads well as-is. Anything that parses as a
/// number (`42.5`, `-90`, `1e-3`) is replaced by a formatted axis value;
/// dates, times and other text are kept.
pub(super) fn is_display_coord(coord: &str) -> bool {
    let coord = coord.trim();
    !coord.is_empty() && coord.parse::<f64>().is_err()
}
