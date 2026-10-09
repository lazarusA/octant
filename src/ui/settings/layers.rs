//! The Layers section: the base layer, then each overlay with its visibility,
//! colormap, opacity, order and remove button.

use crate::app::OctantApp;
use crate::app::layers::{Layer, LayerId};
use crate::ui::icons::{Icon, IconSize, ToolbarButton, UiIconExt};

/// What a row asked for this frame, applied after the list is drawn.
enum RowAction {
    ToggleVisible(LayerId),
    Move(LayerId, bool),
    Remove(LayerId),
}

/// Draws the section while any overlay exists.
pub fn show_layer_list(app: &mut OctantApp, ui: &mut egui::Ui) {
    if app.layers.overlays().is_empty() {
        return;
    }
    super::section(ui, "Layers");
    let count = app.layers.overlays().len();
    let mut action = None;
    // Topmost first, as they stack on the canvas.
    for (i, id) in app.layers.overlay_ids().into_iter().enumerate().rev() {
        ui.push_id(("layer_row", id), |ui| {
            if let Some(a) = overlay_row(app, ui, id, i, count) {
                action = Some(a);
            }
        });
        ui.add_space(2.0);
    }
    base_row(app, ui);
    match action {
        Some(RowAction::ToggleVisible(id)) => {
            if let Some(layer) = app.layers.get_mut(id) {
                layer.visible = !layer.visible;
            }
        }
        Some(RowAction::Move(id, up)) => {
            app.layers.move_overlay(id, up);
        }
        Some(RowAction::Remove(id)) => app.remove_overlay(id),
        None => {}
    }
}

/// The base layer: its variable name, muted; it decides the canvas.
fn base_row(app: &OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add_space(ui.spacing().interact_size.y);
        let name = layer_name(&app.layers.base);
        ui.label(egui::RichText::new(name).weak())
            .on_hover_text("Base layer: decides the canvas, axes and view");
    });
}

/// Overlay `id` (`index` of `count` in drawing order): name row with its
/// controls, why it is not drawn, then colormap and opacity.
fn overlay_row(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    index: usize,
    count: usize,
) -> Option<RowAction> {
    let layer = app.layers.get(id)?;
    let reason = layer.alignment.reason();
    let action = name_row(ui, layer, index, count);
    if let Some(reason) = reason {
        super::note(ui, reason);
    }
    ui.horizontal(|ui| {
        ui.add_space(ui.spacing().interact_size.y);
        crate::ui::colormap::show_layer_colormap_button(app, ui, id);
        if let Some(layer) = app.layers.get_mut(id) {
            ui.add(
                egui::Slider::new(&mut layer.color.opacity, 0.0..=1.0)
                    .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
            )
            .on_hover_text("Opacity");
        }
    });
    egui::CollapsingHeader::new("Color")
        .id_salt(("layer_color_menu", id))
        .default_open(false)
        .show(ui, |ui| super::show_color_menu(app, ui, id));
    action
}

/// The eye toggle, the name, and the move and remove buttons of `layer`.
fn name_row(ui: &mut egui::Ui, layer: &Layer, index: usize, count: usize) -> Option<RowAction> {
    let id = layer.id();
    let mut action = None;
    ui.horizontal(|ui| {
        let (icon, hover) = if layer.visible {
            (Icon::Eye, "Hide layer")
        } else {
            (Icon::EyeOff, "Show layer")
        };
        if ui.add(small(icon, hover)).clicked() {
            action = Some(RowAction::ToggleVisible(id));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.close_button("Remove layer").clicked() {
                action = Some(RowAction::Remove(id));
            }
            let moves = [
                (index > 0, Icon::ChevronDown, "Move down", false),
                (index + 1 < count, Icon::ChevronUp, "Move up", true),
            ];
            for (enabled, icon, hover, up) in moves {
                if ui.add_enabled(enabled, small(icon, hover)).clicked() {
                    action = Some(RowAction::Move(id, up));
                }
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let name = egui::RichText::new(layer_name(layer)).strong();
                ui.add(egui::Label::new(name).truncate());
            });
        });
    });
    action
}

/// The layer's variable name, or a placeholder before its metadata.
fn layer_name(layer: &Layer) -> &str {
    layer
        .selection()
        .variable_info()
        .map_or("(no variable)", |v| v.leaf_name())
}

fn small<'a>(icon: Icon, hover: &'a str) -> ToolbarButton<'a> {
    ToolbarButton::new(icon, hover)
        .compact(true)
        .icon_size(IconSize::Sm)
}
