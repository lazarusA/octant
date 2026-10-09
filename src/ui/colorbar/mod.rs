//! Floating colorbars: one panel per drawn layer (`panel`), stacked upward
//! from the base layer's above the bottom bar.

pub mod bars;
pub mod checker;
pub mod handles;
mod panel;
#[cfg(test)]
mod tests;
pub mod ticks;

pub use ticks::{ColorbarTick, format_scientific_tick, generate_colorbar_ticks};

use crate::app::OctantApp;
use crate::app::layers::LayerId;

/// Shows a colorbar for the base layer and each drawn overlay, unless the
/// layer is an RGB composite (it has no colormap).
pub fn show_colorbar_overlay(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_colorbar {
        return;
    }
    let placement = panel::Placement::of(app, ctx);
    let mut slot = 0;
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
        panel::show(app, ctx, id, placement.slot(slot));
        slot += 1;
    }
}

/// Egui id salt of layer `id`'s colorbar widgets.
fn salt(what: &'static str, id: LayerId) -> (&'static str, LayerId) {
    (what, id)
}
