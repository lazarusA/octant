//! Adding, removing and reordering overlays, and keeping them on the base
//! layer's grid and window as the base plot changes.

use crate::app::OctantApp;
use crate::app::layers::{LayerId, Source, VariableSelection, classify, overlay_selection};
use crate::ui::toast::Severity;

/// Most overlays drawn over the base layer (the hover card has a row each).
pub const MAX_OVERLAYS: usize = 4;
/// Opacity a new overlay starts at, so the base shows through.
const OVERLAY_OPACITY: f32 = 0.75;

impl OctantApp {
    /// Why no overlay can be added right now; `None` when one can.
    pub fn overlay_unavailable(&self) -> Option<&'static str> {
        if self.plotted().metadata.is_none() {
            return Some("Plot a variable first");
        }
        if self.effective_canvas_plot_type() != crate::plots::PlotType::Heatmap {
            return Some("Overlays draw over heatmaps");
        }
        if self.layers.overlays().len() >= MAX_OVERLAYS {
            return Some("At most 4 overlays");
        }
        None
    }

    /// Why variable `var_idx` of the staged dataset can't be added as an
    /// overlay: any reason none can, or it already is a layer (the plot or an
    /// overlay; each variable overlays once). `None` when it can.
    pub fn overlay_unavailable_for(&self, var_idx: usize) -> Option<&'static str> {
        self.overlay_unavailable().or_else(|| {
            let staged = &self.selected;
            let shown = self.layers.iter().any(|layer| {
                let s = layer.selection();
                s.variable_idx == var_idx
                    && s.store_kind == staged.store_kind
                    && s.store_target == staged.store_target
                    && s.metadata.is_some()
            });
            shown.then_some("Already a layer of the plot")
        })
    }

    /// Adds variable `var_idx` of the staged dataset as an overlay reading the
    /// base layer's window, and loads it. Tells the user when it can't be.
    pub fn add_overlay(&mut self, var_idx: usize) -> Option<LayerId> {
        if let Some(reason) = self.overlay_unavailable_for(var_idx) {
            self.notify(Severity::Warning, "Can't add overlay", reason);
            return None;
        }
        let selection = overlay_selection(self.plotted(), &self.selected, var_idx)?;
        let name = selection
            .variable_info()
            .map(|v| v.name.clone())
            .unwrap_or_default();
        let alignment = classify(self.plotted(), &selection);
        if let Some(reason) = alignment.reason() {
            self.notify(
                Severity::Warning,
                "Can't add overlay",
                format!("'{name}': {reason}"),
            );
            return None;
        }
        let var = selection.variable_info().cloned();
        let id = self.layers.push(Source::Variable(selection));
        if let Some(layer) = self.layers.get_mut(id) {
            layer.alignment = alignment;
            layer.color.opacity = OVERLAY_OPACITY;
        }
        if let Some(var) = var {
            crate::ui::variables_panel::init_layer_composite_defaults(self, id, &var);
        }
        self.load_layer_block(id);
        Some(id)
    }

    /// The Dimensions panel's Plot Data button: while its Add Overlay toggle
    /// is on, adds the staged variable as an overlay and turns the toggle off;
    /// otherwise plots the staged selection in place of the base layer and
    /// opens the settings.
    pub fn plot_from_panel(&mut self) {
        if self.plot_as_overlay {
            self.plot_as_overlay = false;
            self.add_overlay(self.selected.variable_idx);
            return;
        }
        self.show_hero = false;
        self.plot_selection();
        self.open_only_settings_panel();
    }

    /// Removes overlay `id`, its opacity curve row and the colormap picker's
    /// hold on it.
    pub fn remove_overlay(&mut self, id: LayerId) {
        if !self.layers.remove(id) {
            return;
        }
        crate::utils::colormap::registry::set_alpha_curve(id.key(), None);
        if self.colormaps.target == Some(id) {
            self.colormaps.target = None;
        }
    }

    /// Re-derives every overlay's window from the base layer's (after the
    /// base plot changed), reloading those whose window moved, then updates
    /// their alignment. The animated index alone (a playback step) reloads
    /// nothing: `load_step_blocks` does that.
    pub(crate) fn sync_overlays_to_base(&mut self) {
        for id in self.layers.overlay_ids() {
            let Some(layer) = self.layers.get(id) else {
                continue;
            };
            let current = layer.selection();
            let Some(next) = overlay_selection(self.plotted(), current, current.variable_idx)
            else {
                continue;
            };
            if !window_differs(current, &next) {
                continue;
            }
            if let Some(layer) = self.layers.get_mut(id) {
                *layer.selection_mut() = next;
                layer.clear_2d();
            }
            self.load_layer_block(id);
        }
        self.refresh_alignments();
    }

    /// Classifies every overlay against the base layer again (its plot or the
    /// coordinates of either side changed).
    pub(crate) fn refresh_alignments(&mut self) {
        for id in self.layers.overlay_ids() {
            let Some(layer) = self.layers.get(id) else {
                continue;
            };
            let alignment = classify(self.layers.base.selection(), layer.selection());
            if let Some(layer) = self.layers.get_mut(id) {
                layer.alignment = alignment;
            }
        }
    }
}

/// Whether `next` reads another window than `current`: other roles, ranges,
/// or fixed indices (the animated one aside).
fn window_differs(current: &VariableSelection, next: &VariableSelection) -> bool {
    let roles = |s: &VariableSelection| {
        s.dim_config
            .iter()
            .map(|c| (c.spatial, c.animation, c.active))
            .collect::<Vec<_>>()
    };
    let fixed = |s: &VariableSelection| {
        s.dim_indices
            .iter()
            .enumerate()
            .filter(|&(i, _)| Some(i) != s.animated_dim)
            .map(|(_, &index)| index)
            .collect::<Vec<_>>()
    };
    current.dim_ranges != next.dim_ranges
        || roles(current) != roles(next)
        || fixed(current) != fixed(next)
}
