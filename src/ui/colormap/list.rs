//! Virtualized colormap list: one clickable row (swatch, name, family) per match.

use super::search::family_label;
use super::swatch::SwatchAtlas;
use crate::app::OctantApp;
use crate::utils::colormap::registry;
use egui::{Align2, FontId, Rect, Sense, vec2};

const ROW_HEIGHT: f32 = 22.0;
const LIST_HEIGHT: f32 = 280.0;
const SWATCH_WIDTH: f32 = 96.0;

pub fn show(app: &mut OctantApp, ui: &mut egui::Ui) {
    let active = app.active_colormap;
    let reversed = app.colormaps.reversed;
    let picker = &mut app.colormaps.picker;
    let ids = picker.cache.ids(&picker.filter);
    if ids.is_empty() {
        ui.label(egui::RichText::new("No colormaps match").small().weak());
        return;
    }

    let mut hovered = None;
    let mut clicked = None;
    egui::ScrollArea::vertical()
        .id_salt(("colormap_list", 0))
        .max_height(LIST_HEIGHT)
        .auto_shrink([false, true])
        .show_rows(ui, ROW_HEIGHT, ids.len(), |ui, range| {
            for &id in ids.get(range).unwrap_or_default() {
                let response = row(ui, &picker.swatches, id, id == active, reversed);
                if response.hovered() {
                    hovered = Some(id);
                }
                if response.clicked() {
                    clicked = Some(id);
                }
            }
        });

    if let Some(id) = hovered
        && app.preview_colormap != Some(id)
    {
        app.preview_colormap = Some(id);
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
    let swatch = Rect::from_min_size(
        rect.left_center() + vec2(4.0, -6.0),
        vec2(SWATCH_WIDTH, 12.0),
    );
    swatches.paint(ui, swatch, id, reversed);

    let font = FontId::proportional(13.0);
    let small = FontId::proportional(11.0);
    let text_color = visuals.text_color();
    let weak = ui.visuals().weak_text_color();
    registry::with_entry(id, |e| {
        let name_pos = egui::pos2(swatch.max.x + 8.0, rect.center().y);
        ui.painter()
            .text(name_pos, Align2::LEFT_CENTER, &e.name, font, text_color);
        let family_pos = rect.right_center() - vec2(6.0, 0.0);
        ui.painter().text(
            family_pos,
            Align2::RIGHT_CENTER,
            family_label(e.family),
            small,
            weak,
        );
    });
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
