//! The Layers menu: one entry per layer, topmost overlay first and the base
//! layer last. Each has its header (visibility, order and remove for
//! overlays), colormap and opacity, composite channels and its own Color menu.

use crate::app::OctantApp;
use crate::app::layers::{Layer, LayerId};
use crate::plots::PlotType;
use crate::ui::icons::{Icon, IconSize, ToolbarButton, UiIconExt};

/// What a row asked for this frame, applied after the list is drawn.
enum RowAction {
    ToggleVisible(LayerId),
    Move(LayerId, bool),
    Remove(LayerId),
}

/// The collapsible Layers menu.
pub fn show_layers_menu(app: &mut OctantApp, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Layers")
        .id_salt("settings_layers_section")
        .default_open(false)
        .show(ui, |ui| show_layer_entries(app, ui));
}

fn show_layer_entries(app: &mut OctantApp, ui: &mut egui::Ui) {
    let ids = app.layers.overlay_ids();
    let count = ids.len();
    let mut action = None;
    // Topmost first, as they stack on the canvas.
    for (i, &id) in ids.iter().enumerate().rev() {
        ui.push_id(("layer_entry", id), |ui| {
            if let Some(a) = overlay_header(app, ui, id, i, count) {
                action = Some(a);
            }
            layer_body(app, ui, id);
        });
        ui.separator();
    }
    ui.push_id(("layer_entry", LayerId::BASE), |ui| {
        base_header(app, ui);
        layer_body(app, ui, LayerId::BASE);
    });
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

/// The base layer's name; it decides the canvas.
fn base_header(app: &OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let name = layer_name(app, LayerId::BASE);
        ui.add(egui::Label::new(egui::RichText::new(name).strong()).truncate())
            .on_hover_text("Base layer: decides the canvas, axes and view");
        ui.label(egui::RichText::new("base").small().weak());
    });
}

/// Overlay `id`'s header (`index` of `count` in drawing order) and why it
/// is not drawn.
fn overlay_header(
    app: &OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    index: usize,
    count: usize,
) -> Option<RowAction> {
    let layer = app.layers.get(id)?;
    let action = name_row(app, ui, layer, index, count);
    if let Some(reason) = layer.alignment.reason() {
        super::note(ui, reason);
    }
    action
}

/// Layer `id`'s colormap and opacity, composite channels and Color menu.
fn layer_body(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    ui.horizontal(|ui| {
        crate::ui::colormap::show_layer_colormap_button(app, ui, id);
        let opacity = &mut app.layers.get_or_base_mut(id).color.opacity;
        ui.add(
            egui::Slider::new(opacity, 0.0..=1.0)
                .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
        )
        .on_hover_text("Opacity");
    });
    // A line plot draws one band: no composite.
    let line = id == LayerId::BASE && app.effective_canvas_plot_type() == PlotType::Line;
    if !line {
        super::composite::show_composite_controls(app, ui, id);
    }
    egui::CollapsingHeader::new("Color")
        .id_salt(("layer_color_menu", id))
        .default_open(false)
        .show(ui, |ui| super::show_color_menu(app, ui, id));
}

/// The eye toggle, the name, and the move and remove buttons of `layer`.
fn name_row(
    app: &OctantApp,
    ui: &mut egui::Ui,
    layer: &Layer,
    index: usize,
    count: usize,
) -> Option<RowAction> {
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
                let name = egui::RichText::new(layer_name(app, id)).strong();
                ui.add(egui::Label::new(name).truncate());
            });
        });
    });
    action
}

/// Layer `id`'s variable name (the base layer's staged one before its first
/// plot), or a placeholder.
fn layer_name(app: &OctantApp, id: LayerId) -> &str {
    app.layer_variable_info(id)
        .map_or("(no variable)", |v| v.leaf_name())
}

fn small<'a>(icon: Icon, hover: &'a str) -> ToolbarButton<'a> {
    ToolbarButton::new(icon, hover)
        .compact(true)
        .icon_size(IconSize::Sm)
}
