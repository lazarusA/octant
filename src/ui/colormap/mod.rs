//! Colormap picker: toolbar popup with search, kind/family filters, gradient
//! swatches for every registered colormap, reverse and smooth toggles, and the
//! custom colormap editor.

pub mod editor;
pub mod filters;
pub mod label;
pub mod list;
pub mod search;
pub mod swatch;

#[cfg(test)]
mod tests;

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::ui::icons::{Icon, ToolbarButton, UiIconExt};
use crate::utils::colormap::registry;

/// Persistent (per-session) picker UI state.
#[derive(Default)]
pub struct PickerState {
    pub filter: search::FilterKey,
    pub cache: search::FilterCache,
    pub swatches: swatch::SwatchAtlas,
    pub editor: editor::EditorState,
    /// Height of everything below the list (editor, colorbar options) last frame.
    pub below_list_height: f32,
    /// Row hovered last frame; `preview_colormap` itself resets every frame.
    pub last_hovered: Option<u32>,
}

const POPUP_WIDTH: f32 = 300.0;
/// Room left below the content: the popup frame's padding and border (6 + 1 px,
/// top and bottom) plus 2 px so the panel never touches the window edge.
const POPUP_MARGIN: f32 = 16.0;
const MIN_POPUP_HEIGHT: f32 = 160.0;
const ACTIVE_SWATCH_WIDTH: f32 = 120.0;
const LAYER_SWATCH_WIDTH: f32 = 72.0;

pub fn show_colormap_menu(app: &mut OctantApp, ui: &mut egui::Ui, compact: bool) {
    let button_response = ui.add(
        ToolbarButton::new(Icon::Colormap, "Colormap")
            .compact(compact)
            .owns_popup(),
    );
    if button_response.clicked() {
        // The toolbar's picker edits the base layer.
        app.colormaps.target = None;
    }
    // Pinned below the button: egui otherwise re-picks the side every frame from
    // the content size, so expanding the editor would jump the panel right or up.
    // The height is capped to the space below the button and scrolls instead.
    let viewport = ui.ctx().input(|i| i.viewport_rect());
    let max_height =
        (viewport.max.y - button_response.rect.max.y - POPUP_MARGIN).max(MIN_POPUP_HEIGHT);
    egui::Popup::from_toggle_button_response(&button_response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .align(egui::RectAlign::BOTTOM_START)
        .align_alternatives(&[])
        .show(|ui| {
            panel_scroll_area(max_height)
                .show(ui, |ui| render_colormap_contents(app, ui, max_height));
        });
}

/// Swatch button showing layer `id`'s colormap; it opens the picker on that
/// layer (`ColormapState::target`).
pub fn show_layer_colormap_button(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    let Some(style) = app.layers.get(id).map(|l| &l.color) else {
        return;
    };
    let (shown, reversed) = (style.shown_row(style.colormap), style.reversed);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(LAYER_SWATCH_WIDTH, 14.0), egui::Sense::click());
    app.colormaps.picker.swatches.ensure(ui.ctx());
    app.colormaps
        .picker
        .swatches
        .paint(ui, rect, shown, reversed);
    if response.hovered() {
        let stroke = ui.visuals().widgets.hovered.fg_stroke;
        ui.painter()
            .rect_stroke(rect, 2.0, stroke, egui::StrokeKind::Outside);
    }
    let response = response.on_hover_text("Colormap");
    if response.clicked() {
        app.colormaps.target = Some(id);
    }
    let viewport = ui.ctx().input(|i| i.viewport_rect());
    let max_height = (viewport.max.y - response.rect.max.y - POPUP_MARGIN).max(MIN_POPUP_HEIGHT);
    egui::Popup::from_toggle_button_response(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .align(egui::RectAlign::BOTTOM_START)
        .align_alternatives(&[])
        .show(|ui| {
            panel_scroll_area(max_height)
                .show(ui, |ui| render_colormap_contents(app, ui, max_height));
        });
}

/// Scroll area of the whole panel. It grows with the content and scrolls only
/// when the content does not fit in `max_height`.
///
/// A popup's `Area` offers its content only last frame's size, so a plain
/// scroll area could never grow past it: when the editor opened, the panel kept
/// its old height and scrolled. `min_scrolled_height` lets it take up to
/// `max_height`, and auto-shrink trims it back to the content.
fn panel_scroll_area(max_height: f32) -> egui::ScrollArea {
    egui::ScrollArea::vertical()
        .id_salt(("colormap_popup_scroll", 0))
        .max_height(max_height)
        .min_scrolled_height(max_height)
}

/// Lays out the panel so everything fits in `max_height`: the colormap list
/// takes whatever height is left after the controls above it and the editor and
/// colorbar options below it (measured on the previous frame).
fn render_colormap_contents(app: &mut OctantApp, ui: &mut egui::Ui, max_height: f32) {
    ui.set_width(POPUP_WIDTH);
    let top = ui.cursor().min.y;
    app.colormaps.picker.swatches.ensure(ui.ctx());

    show_active_row(app, ui);
    if registry::smooth_variant(app.picker_style().colormap).is_some() {
        ui.checkbox(&mut app.picker_style_mut().smooth, "Smooth")
            .on_hover_text("Blend this palette's colors into a continuous gradient (Oklab)");
    }
    ui.separator();

    ui.search_field(
        &mut app.colormaps.picker.filter.query,
        "Search colormaps...",
        None,
    );
    filters::show(ui, &mut app.colormaps.picker.filter);
    ui.add_space(2.0);
    let used_above = ui.cursor().min.y - top;
    let below = app.colormaps.picker.below_list_height;
    // The list is followed by one item spacing before the content below it.
    let spacing = ui.spacing().item_spacing.y;
    let list_height =
        (max_height - used_above - spacing - below).clamp(list::MIN_HEIGHT, list::MAX_HEIGHT);
    list::show(app, ui, list_height);

    let below_start = ui.cursor().min.y;
    ui.separator();
    editor::show(app, ui);
    ui.separator();
    show_colorbar_options(app, ui);
    let measured = ui.cursor().min.y - below_start;
    if (measured - below).abs() > 0.5 {
        // The editor opened or closed: resize the list on the next frame.
        app.colormaps.picker.below_list_height = measured;
        ui.ctx().request_repaint();
    }
}

/// Active colormap swatch, its (elided) name and the reverse toggle.
fn show_active_row(app: &mut OctantApp, ui: &mut egui::Ui) {
    let id = app.picker_style().colormap;
    let reversed = app.picker_style().reversed;
    ui.horizontal(|ui| {
        let (rect, swatch_response) =
            ui.allocate_exact_size(egui::vec2(ACTIVE_SWATCH_WIDTH, 14.0), egui::Sense::hover());
        let shown = app.shown_colormap(id);
        app.colormaps
            .picker
            .swatches
            .paint(ui, rect, shown, reversed);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(&mut app.picker_style_mut().reversed, "Reversed");
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let font_size = egui::TextStyle::Body.resolve(ui.style()).size;
                let max_width = ui.available_width();
                let Some(galley) = label::name_galley(ui.painter(), id, font_size, max_width)
                else {
                    return;
                };
                let (rect, name_response) =
                    ui.allocate_exact_size(galley.size(), egui::Sense::hover());
                let color = ui.visuals().strong_text_color();
                ui.painter().galley(rect.min, galley, color);
                name_response.union(swatch_response).on_hover_ui(|ui| {
                    registry::with_entry(id, |e| label::hover_details(ui, e));
                });
            });
        });
    });
}

fn show_colorbar_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.checkbox(&mut app.show_colorbar, "Show Colorbar");
    if app.show_colorbar {
        ui.add(
            egui::Slider::new(&mut app.colorbar_transparency, 0.0..=1.0)
                .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
        );
        ui.label(egui::RichText::new("Box transparency"));
    }
}

/// Makes `id` the picked layer's colormap, leaving RGB composite mode if needed.
pub fn select_colormap(app: &mut OctantApp, id: u32) {
    let layer = app.picker_layer();
    if let Some(l) = app.layers.get_mut(layer)
        && l.composite.enabled
    {
        l.composite.enabled = false;
        app.load_layer_block(layer);
    }
    app.picker_style_mut().colormap = id;
    app.preview_colormap = None;
}
