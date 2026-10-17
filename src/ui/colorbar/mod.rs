//! Floating colorbars: one panel per drawn layer (`panel`), the base
//! layer's above the canvas bottom and each overlay's in its own slot
//! (`layout`), horizontal or vertical (`axis`), moved and flipped by its
//! controls (`controls`). Clicking the bar opens the layer's colormap
//! picker, the cog its Color window (`color_menu`).

mod axis;
pub mod bars;
pub mod checker;
mod color_menu;
#[cfg(test)]
mod color_menu_tests;
mod controls;
#[cfg(test)]
mod controls_tests;
pub mod handles;
mod layout;
#[cfg(test)]
mod layout_tests;
#[cfg(test)]
mod order_tests;
mod panel;
mod series;
#[cfg(test)]
mod tests;
mod tick_label;
pub mod ticks;

pub use ticks::{ColorbarTick, format_scientific_tick, generate_colorbar_ticks};

use crate::app::OctantApp;
use crate::app::layers::LayerId;

/// Shows a colorbar on `canvas` for the base layer and each drawn overlay
/// that has one (`can_show`) and has not hidden it.
pub fn show_colorbar_overlay(app: &mut OctantApp, ctx: &egui::Context, canvas: egui::Rect) {
    for i in 0..=app.layers.overlays().len() {
        let layer = match i {
            0 => Some(&app.layers.base),
            n => app.layers.overlays().get(n - 1),
        };
        let Some(id) = layer
            .filter(|l| l.is_drawn() && l.colorbar.shown && can_show(app, l.id()))
            .map(|l| l.id())
        else {
            continue;
        };
        panel::show(app, ctx, id, canvas);
    }
}

/// Whether layer `id` has a colorbar at all: RGB composites and label
/// images draw their own colors, as does the base layer's line plot in its
/// custom color. Their Color menu stays in Settings' Layers menu.
pub fn can_show(app: &OctantApp, id: LayerId) -> bool {
    let custom_line = id == LayerId::BASE && app.line_custom_colored();
    app.layers
        .get(id)
        .is_some_and(|l| !l.composite.enabled && l.labels().is_none() && !custom_line)
}

/// Egui id salt of layer `id`'s colorbar widgets.
fn salt(what: &'static str, id: LayerId) -> (&'static str, LayerId) {
    (what, id)
}
