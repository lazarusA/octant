//! Adding, removing and reordering overlays, and keeping them on the base
//! layer's grid and window as the base plot changes.

use crate::app::OctantApp;
use crate::app::layers::{
    Alignment, LayerId, Source, classify, follow_base_window, overlay_selection,
};
use crate::ui::toast::Severity;
use crate::utils::colormap::registry;

/// The overlay limit as a literal, so its message can name it.
macro_rules! max_overlays {
    () => {
        4
    };
}

/// Most overlays drawn over the base layer (the hover card has a row each).
pub const MAX_OVERLAYS: usize = max_overlays!();
/// Why no further overlay can be added once [`MAX_OVERLAYS`] exist.
const STACK_FULL: &str = concat!("At most ", max_overlays!(), " overlays");
/// Opacity a new overlay starts at, so the base shows through.
const OVERLAY_OPACITY: f32 = 0.75;
/// Colormaps new overlays take, in order: perceptually uniform sequential
/// maps in hue families apart from the default viridis and from each other.
pub const OVERLAY_COLORMAPS: [&str; 8] = [
    "matplotlib:magma",
    "cmocean:ice",
    "cmocean:algae",
    "cmocean:amp",
    "scientific:lajolla",
    "cmocean:haline",
    "scientific:oslo",
    "cmocean:matter",
];

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
            return Some(STACK_FULL);
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
        let name = selection.variable_info().map_or("", |v| v.name.as_str());
        let alignment = classify(self.plotted(), &selection);
        // Another grid or none at all can't be drawn; an overlay whose
        // coordinates haven't arrived yet is added and drawn once they match.
        if let Some(reason) = alignment
            .reason()
            .filter(|_| alignment != Alignment::IndexOnly)
        {
            let detail = format!("'{name}': {reason}");
            self.notify(Severity::Warning, "Can't add overlay", detail);
            return None;
        }
        let var = selection.variable_info().cloned();
        let id = self.layers.push(Source::Variable(selection));
        let colormap = self.next_overlay_colormap();
        if let Some(layer) = self.layers.get_mut(id) {
            layer.alignment = alignment;
            layer.color.opacity = OVERLAY_OPACITY;
            layer.color.colormap = colormap;
        }
        if let Some(var) = var {
            crate::ui::variables_panel::init_layer_composite_defaults(self, id, &var);
        }
        if alignment.is_drawn() {
            self.load_layer_block(id);
        } else {
            self.request_layer_coordinates(id);
        }
        Some(id)
    }

    /// The colormap a new overlay takes: the first of [`OVERLAY_COLORMAPS`]
    /// no layer draws yet, cycling by the overlay count once all are taken.
    fn next_overlay_colormap(&self) -> u32 {
        let ids = OVERLAY_COLORMAPS
            .iter()
            .filter_map(|key| registry::find(key));
        let mut unused = ids
            .clone()
            .filter(|&id| self.layers.iter().all(|l| l.color.colormap != id));
        let fallback = ids
            .cycle()
            .nth(self.layers.overlays().len())
            .unwrap_or_else(registry::default_id);
        unused.next().unwrap_or(fallback)
    }

    /// The Dimensions panel's Plot Data button: while its Add Overlay toggle
    /// is on, adds the staged variable as an overlay and turns the toggle off;
    /// otherwise plots the staged selection in place of the base layer and
    /// opens the settings.
    pub fn plot_from_panel(&mut self) {
        // The Layers menu shows what was plotted or added.
        self.layout.reveal_layers_menu = true;
        if self.layout.plot_as_overlay {
            self.layout.plot_as_overlay = false;
            self.add_overlay(self.selected.variable_idx);
            return;
        }
        self.layout.show_hero = false;
        self.plot_selection();
        self.open_only_settings_panel();
    }

    /// Removes overlay `id`, its opacity curve row and the colormap picker's
    /// hold on it (its target and any preview aimed at it).
    pub fn remove_overlay(&mut self, id: LayerId) {
        if !self.layers.remove(id) {
            return;
        }
        registry::set_alpha_curve(id.key(), None);
        if self.colormaps.target == Some(id) {
            self.colormaps.target = None;
            self.preview_colormap = None;
        }
    }

    /// Shows or hides overlay `id`; a layer shown again loads the current
    /// step, which hidden layers skip.
    pub fn set_layer_visible(&mut self, id: LayerId, visible: bool) {
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        layer.visible = visible;
        if layer.is_drawn() {
            self.load_layer_block(id);
        }
    }

    /// Gives every overlay the base layer's window again (after the base plot
    /// changed) and classifies it; those whose window moved reload when drawn.
    /// The animated index alone (a playback step) reloads nothing:
    /// `load_step_blocks` does that.
    pub(crate) fn sync_overlays_to_base(&mut self) {
        for id in self.layers.overlay_ids() {
            let Some((base, layer)) = self.layers.base_and_overlay_mut(id) else {
                continue;
            };
            let moved = follow_base_window(base.selection(), layer.selection_mut());
            layer.alignment = classify(base.selection(), layer.selection());
            if moved {
                layer.clear_2d();
                if layer.is_drawn() {
                    self.load_layer_block(id);
                }
            }
        }
    }

    /// Classifies every overlay against the base layer again (its plot or the
    /// coordinates of either side changed); an overlay drawn from now on
    /// loads its block.
    pub(crate) fn refresh_alignments(&mut self) {
        for id in self.layers.overlay_ids() {
            let Some((base, layer)) = self.layers.base_and_overlay_mut(id) else {
                continue;
            };
            let was_drawn = layer.is_drawn();
            layer.alignment = classify(base.selection(), layer.selection());
            if layer.is_drawn() && !was_drawn {
                self.load_layer_block(id);
            }
        }
    }
}
