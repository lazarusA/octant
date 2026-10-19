//! Colormap picker: the popup a colorbar's bar opens, with search,
//! kind/family filters, gradient swatches for every registered colormap,
//! reverse and smooth toggles, and the custom colormap editor.

pub mod editor;
pub mod filters;
pub mod keys;
pub mod label;
pub mod list;
pub mod search;
pub mod swatch;

#[cfg(test)]
mod tests;

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::i18n::{ColormapKey as K, tr};
use crate::ui::icons::UiIconExt;
use crate::ui::keyboard::list_nav::Above;
use crate::ui::keyboard::windows;
use crate::utils::colormap::registry;

/// Persistent (per-session) picker UI state.
#[derive(Default)]
pub struct PickerState {
    pub filter: search::FilterKey,
    pub cache: search::FilterCache,
    pub swatches: swatch::SwatchAtlas,
    pub editor: editor::EditorState,
    /// Height of everything below the list (the editor) last frame.
    pub below_list_height: f32,
    /// Row hovered last frame; `preview_colormap` itself resets every frame.
    pub last_hovered: Option<u32>,
    /// Row holding keyboard focus last frame.
    pub focused_row: Option<u32>,
    /// Rows the list drew last frame.
    pub drawn_rows: std::ops::Range<usize>,
    /// Focus the search field on the next frame (the picker just opened).
    pub focus_search: bool,
}

const POPUP_WIDTH: f32 = 300.0;
/// Room left below the content: the popup frame's padding and border (6 + 1 px,
/// top and bottom) plus 2 px so the panel never touches the window edge.
const POPUP_MARGIN: f32 = 16.0;
const MIN_POPUP_HEIGHT: f32 = 160.0;
const ACTIVE_SWATCH_WIDTH: f32 = 120.0;

/// The picker for layer `id`, opened and closed by clicking `anchor` (its
/// colorbar's bar): below it when the bar sits in the upper half of the
/// window, else above it.
pub fn show_picker_popup(app: &mut OctantApp, anchor: &egui::Response, id: LayerId) {
    if anchor.clicked() {
        app.colormaps.target = Some(id);
        app.colormaps.picker.focus_search = true;
    }
    // Focus returns to the bar once the picker closes.
    let popup_id = egui::Popup::default_response_id(anchor);
    windows::follow_window(
        &anchor.ctx,
        popup_id,
        egui::Popup::is_id_open(&anchor.ctx, popup_id),
    );
    // The side follows the bar, not the content: egui otherwise re-picks it
    // every frame from the content size, so expanding the editor would jump
    // the panel. The height is capped to the room on that side and scrolls.
    let viewport = anchor.ctx.input(|i| i.viewport_rect());
    let below = anchor.rect.center().y < viewport.center().y;
    let (align, room) = if below {
        (
            egui::RectAlign::BOTTOM_START,
            viewport.max.y - anchor.rect.max.y,
        )
    } else {
        (
            egui::RectAlign::TOP_START,
            anchor.rect.min.y - viewport.min.y,
        )
    };
    let max_height = (room - POPUP_MARGIN).max(MIN_POPUP_HEIGHT);
    egui::Popup::from_toggle_button_response(anchor)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .align(align)
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
/// takes whatever height is left after the controls above it and the editor
/// below it (measured on the previous frame).
fn render_colormap_contents(app: &mut OctantApp, ui: &mut egui::Ui, max_height: f32) {
    ui.set_width(POPUP_WIDTH);
    let top = ui.cursor().min.y;
    app.colormaps.picker.swatches.ensure(ui.ctx());

    show_active_row(app, ui);
    if registry::smooth_variant(app.picker_style().colormap).is_some()
        && let Some(style) = app.picker_style_mut()
    {
        ui.checkbox(&mut style.smooth, tr(K::Smooth))
            .on_hover_text(tr(K::SmoothHover));
    }
    ui.separator();

    let search = ui.search_field_response(
        &mut app.colormaps.picker.filter.query,
        tr(K::SearchHint),
        None,
    );
    if std::mem::take(&mut app.colormaps.picker.focus_search) {
        search.request_focus();
    }
    let above = Above::of(&search);
    filters::show(ui, &mut app.colormaps.picker.filter);
    ui.add_space(2.0);
    let used_above = ui.cursor().min.y - top;
    let below = app.colormaps.picker.below_list_height;
    // The list is followed by one item spacing before the content below it.
    let spacing = ui.spacing().item_spacing.y;
    let list_height =
        (max_height - used_above - spacing - below).clamp(list::MIN_HEIGHT, list::MAX_HEIGHT);
    list::show(app, ui, list_height, above);

    let below_start = ui.cursor().min.y;
    ui.separator();
    editor::show(app, ui);
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
            if let Some(style) = app.picker_style_mut() {
                ui.checkbox(&mut style.reversed, tr(K::Reversed));
            }
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

/// Makes `id` the picked layer's colormap, leaving RGB composite mode if needed.
pub fn select_colormap(app: &mut OctantApp, id: u32) {
    let layer = app.picker_layer();
    if let Some(l) = app.layers.get_mut(layer)
        && l.composite.enabled
    {
        l.composite.enabled = false;
        app.load_layer_block(layer);
    }
    if let Some(style) = app.picker_style_mut() {
        style.colormap = id;
    }
    app.preview_colormap = None;
}
