//! Selects a variable on a time step its store holds data for: ARCO-ERA5
//! allocates its whole time axis but holds data only for the span its root
//! attributes state (`utils::units::coverage`), and the steps outside it read
//! as the fill value. The staged step moves when the variable is selected (or
//! when its time axis arrives, if the user left the slider alone), so the
//! Dimensions panel shows where the data starts before Plot; Plot and a
//! plotted variable keep whatever step is shown.

use crate::app::OctantApp;
use crate::app::layers::VariableSelection;
use crate::data::VariableInfo;
use crate::utils::units::coverage::{stated_coverage, stated_span, step_inside};

impl OctantApp {
    /// After `var` is selected (its dimension defaults applied): moves its
    /// staged step into the span its attributes state, when it lies outside;
    /// remembers the selection (`PlaybackState::coverage_unchecked`) when its
    /// time axis has not loaded yet, and asks for it: the variable a dataset
    /// opens on has not requested its coordinates. Nothing while the selected
    /// metadata is not `var`'s yet (`load_new_metadata` applies defaults
    /// before it swaps the metadata, then calls this again).
    pub(crate) fn select_inside_time_coverage(&mut self, var: &VariableInfo) {
        let is_var = self.selected.variable_info().is_some_and(|v| {
            v.name == var.name && v.shape == var.shape && v.dimension_names == var.dimension_names
        });
        if !is_var {
            return;
        }
        self.playback.coverage_unchecked = None;
        if awaits_time_axis(&self.selected) {
            self.request_variable_coordinates(self.selected.variable_idx);
            let source_id = self.selection_source_id(&self.selected);
            let step = staged_step(&self.selected);
            self.playback.coverage_unchecked =
                Some(((source_id, self.selected.variable_idx), step));
        } else {
            let from = window_step(&self.selected, self.playback.current_timestep);
            self.move_selection_into_coverage(from);
        }
    }

    /// When coordinates of the dataset `source_id` arrive for the remembered
    /// selection: moves its staged step into the stated span, while it is
    /// still selected, not plotted, and on the step it was selected at (the
    /// user has not moved its slider).
    pub(crate) fn check_selection_against_time_coverage(&mut self, source_id: &str) {
        let Some(((id, var_idx), step)) = self.playback.coverage_unchecked.clone() else {
            return;
        };
        let selected =
            self.selected.variable_idx == var_idx && self.selection_source_id(&self.selected) == id;
        if id != source_id || (selected && awaits_time_axis(&self.selected)) {
            return;
        }
        self.playback.coverage_unchecked = None;
        if selected && self.is_exploring_unplotted_variable() && staged_step(&self.selected) == step
        {
            self.move_selection_into_coverage(window_step(&self.selected, step));
        }
    }

    /// Whether the plotted view's step lies outside the span its variable's
    /// attributes state.
    pub(crate) fn plotted_step_outside_coverage(&self) -> bool {
        let step = self.playback.current_timestep;
        coverage_step(self.layers.base.selection(), step).is_some()
    }

    /// Moves the staged animated step (and its window) from `from` and the
    /// shared step into the stated span; its dimension, new step and last
    /// step when it moved.
    fn move_selection_into_coverage(&mut self, from: usize) -> Option<(usize, usize, usize)> {
        let (dim, step) = coverage_step(&self.selected, from)?;
        let last = last_step(&self.selected, dim, step);
        self.selected.hold_step(dim, step, last);
        self.playback.current_timestep = step;
        Some((dim, step, last))
    }
}

/// The staged index of `selection`'s animated dimension (0 without one).
fn staged_step(selection: &VariableSelection) -> usize {
    selection
        .animated_dim()
        .and_then(|d| selection.dim_config.get(d))
        .map_or(0, |c| c.index)
}

/// The step Plot shows for `selection` from the shared step `current`:
/// clamped into its animated window, which a newly selected variable's
/// defaults put at the first chunk wherever the shared step is.
fn window_step(selection: &VariableSelection, current: usize) -> usize {
    selection
        .animated_dim()
        .and_then(|d| selection.dim_config.get(d))
        .map_or(current, |c| {
            current.clamp(c.range.0, c.range.1.max(c.range.0))
        })
}

/// The last step of `selection`'s dimension `dim`, `fallback` when unknown.
fn last_step(selection: &VariableSelection, dim: usize, fallback: usize) -> usize {
    selection
        .variable_info()
        .and_then(|v| v.shape.get(dim))
        .map_or(fallback, |&n| (n as usize).saturating_sub(1))
}

/// Whether `selection`'s variable states a span and animates time, but its
/// time axis has not loaded yet.
fn awaits_time_axis(selection: &VariableSelection) -> bool {
    let (Some(meta), Some(var), Some(dim)) = (
        selection.metadata.as_ref(),
        selection.variable_info(),
        selection.animated_dim(),
    ) else {
        return false;
    };
    let (start, end) = stated_coverage(var);
    let Some(name) = var.dimension_names.get(dim) else {
        return false;
    };
    (start.is_some() || end.is_some()) && meta.get_dim_coords(Some(&var.name), name).is_none()
}

/// The animated dimension of `selection`'s variable and the step inside its
/// stated coverage to show instead of `current`, when `current` lies outside
/// it. Dates convert in the Gregorian calendar only (`noleap`, `360_day`
/// would land elsewhere).
fn coverage_step(selection: &VariableSelection, current: usize) -> Option<(usize, usize)> {
    let meta = selection.metadata.as_ref()?;
    let var = selection.variable_info()?;
    // Most datasets state no span: done before scanning the variables.
    if stated_coverage(var) == (None, None) {
        return None;
    }
    let dim = selection.animated_dim()?;
    let name = var.dimension_names.get(dim)?;
    let len = usize::try_from(*var.shape.get(dim)?).ok()?;
    let coords = meta.get_dim_coords(Some(&var.name), name)?;
    // Units and calendar come from the dimension's own coordinate variable.
    let axis = meta
        .variables
        .iter()
        .find(|v| v.name.eq_ignore_ascii_case(name))?;
    let units = axis.units.as_deref()?;
    let calendar = axis.attributes.get("calendar").map(|c| c.trim());
    let gregorian = ["standard", "gregorian", "proleptic_gregorian"];
    if !calendar.is_none_or(|c| gregorian.iter().any(|g| c.eq_ignore_ascii_case(g))) {
        return None;
    }
    let span = stated_span(var, units);
    let current = current.min(len.saturating_sub(1));
    let range = selection.dim_config.get(dim)?.range;
    let step = step_inside(coords, len, current, range, span)?;
    Some((dim, step))
}
