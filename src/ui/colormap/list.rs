//! Virtualized colormap list: one clickable row (swatch, elided name) per match;
//! the full name, family, kind and license show on hover.

use super::label;
use super::swatch::SwatchAtlas;
use crate::app::OctantApp;
use crate::utils::colormap::registry;
use egui::{Rect, Sense, vec2};

const ROW_HEIGHT: f32 = 22.0;
/// Rows visible at most; the list scrolls for the rest.
const VISIBLE_ROWS: f32 = 7.0;
/// Tallest the list grows; it shrinks to fit the panel, down to [`MIN_HEIGHT`].
pub const MAX_HEIGHT: f32 = ROW_HEIGHT * VISIBLE_ROWS;
/// A few rows always stay visible.
pub const MIN_HEIGHT: f32 = ROW_HEIGHT * 3.0;
/// Share of a row's content width given to the gradient preview; the name gets the rest.
const SWATCH_SHARE: f32 = 0.75;
const PAD_LEFT: f32 = 4.0;
const PAD_RIGHT: f32 = 6.0;
const GAP: f32 = 8.0;

pub fn show(app: &mut OctantApp, ui: &mut egui::Ui, max_height: f32) {
    let active = app.picker_style().colormap;
    let reversed = app.picker_style().reversed;
    let super::PickerState {
        filter,
        cache,
        swatches,
        ..
    } = &mut app.colormaps.picker;
    let ids = cache.ids(filter);
    if ids.is_empty() {
        ui.label(egui::RichText::new("No colormaps match").small().weak());
        return;
    }

    let mut hovered = None;
    let mut clicked = None;
    egui::ScrollArea::vertical()
        .id_salt(("colormap_list", 0))
        .max_height(max_height)
        .auto_shrink([false, true])
        .show_rows(ui, ROW_HEIGHT, ids.len(), |ui, range| {
            for &id in ids.get(range).unwrap_or_default() {
                let response = row(ui, swatches, id, id == active, reversed);
                if response.hovered() {
                    hovered = Some(id);
                }
                if response.clicked() {
                    clicked = Some(id);
                }
            }
        });

    if hovered.is_some() {
        app.preview_colormap = hovered;
    }
    // Repaint only when the hovered row changes, not every frame it stays hovered.
    if hovered != app.colormaps.picker.last_hovered {
        app.colormaps.picker.last_hovered = hovered;
        ui.ctx().request_repaint();
    }
    if let Some(id) = clicked {
        super::select_colormap(app, id);
    }
}

fn row(
    ui: &mut egui::Ui,
    swatches: &SwatchAtlas,
    id: u32,
    selected: bool,
    reversed: bool,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let visuals = ui.style().interact_selectable(&response, selected);
    if selected || response.hovered() {
        ui.painter().rect_filled(rect, 3.0, visuals.weak_bg_fill);
    }
    let content = (rect.width() - PAD_LEFT - PAD_RIGHT - GAP).max(0.0);
    let swatch = Rect::from_min_size(
        rect.left_center() + vec2(PAD_LEFT, -6.0),
        vec2(content * SWATCH_SHARE, 12.0),
    );
    swatches.paint(ui, swatch, id, reversed);

    let name_x = swatch.max.x + GAP;
    let max_width = rect.max.x - PAD_RIGHT - name_x;
    let text_color = visuals.text_color();
    if let Some(galley) = label::name_galley(ui.painter(), id, label::ROW_FONT_SIZE, max_width) {
        let pos = egui::pos2(name_x, rect.center().y - galley.size().y / 2.0);
        ui.painter().galley(pos, galley, text_color);
    }
    let response = response.on_hover_ui(|ui| {
        registry::with_entry(id, |e| label::hover_details(ui, e));
    });
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
