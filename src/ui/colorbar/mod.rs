//! Floating colorbars: one panel per drawn layer (`panel`), the base
//! layer's above the canvas bottom and each overlay's in its own slot
//! (`layout`), horizontal or vertical (`axis`), moved and flipped by its
//! controls (`controls`).

mod axis;
pub mod bars;
pub mod checker;
mod controls;
#[cfg(test)]
mod controls_tests;
pub mod handles;
mod layout;
#[cfg(test)]
mod layout_tests;
mod panel;
#[cfg(test)]
mod tests;
pub mod ticks;

pub use ticks::{ColorbarTick, format_scientific_tick, generate_colorbar_ticks};

use crate::app::OctantApp;
use crate::app::layers::LayerId;

/// Shows a colorbar on `canvas` for the base layer and each drawn overlay,
/// unless the layer is an RGB composite (it has no colormap).
pub fn show_colorbar_overlay(app: &mut OctantApp, ctx: &egui::Context, canvas: egui::Rect) {
    if !app.show_colorbar {
        return;
    }
    for i in 0..=app.layers.overlays().len() {
        let layer = match i {
            0 => Some(&app.layers.base),
            n => app.layers.overlays().get(n - 1),
        };
        let Some(id) = layer
            .filter(|l| l.is_drawn() && !l.composite.enabled)
            .map(|l| l.id())
        else {
            continue;
        };
        panel::show(app, ctx, id, canvas);
    }
}

/// Egui id salt of layer `id`'s colorbar widgets.
fn salt(what: &'static str, id: LayerId) -> (&'static str, LayerId) {
    (what, id)
}
