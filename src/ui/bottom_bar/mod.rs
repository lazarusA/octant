//! Bottom playback bar. Items collapse to icon-only buttons (data badges
//! hide, the date keeps its icon and details popover) as the window narrows, in [`layout::COLLAPSE_ORDER`], instead of
//! moving into an overflow menu. The timeline slider fills the space between
//! the left and right groups.

mod items;
mod labels;
mod layout;
#[cfg(test)]
mod tests;
mod timeline;

use crate::app::OctantApp;
use crate::ui::icons::{Icon, IconSize, UiIconExt};
use crate::ui::toolbar::ItemWidths;
use layout::{ITEM_COUNT, LEFT_ITEMS, RIGHT_ITEMS, SLIDER_MIN_W};
use timeline::Timeline;

pub fn show_bottom_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    if !app.has_animated_dimension() {
        return;
    }

    // Local copy avoids holding `&mut app.layout.show_bottom_bar` while the panel
    // closure borrows all of `app`.
    let mut expanded = app.layout.show_bottom_bar;
    let mut request_expand = false;

    egui::Panel::show_switched(
        ui,
        &mut expanded,
        egui::Panel::bottom("octant_bottom_bar_collapsed")
            .resizable(true)
            .default_size(20.0)
            .size_range(20.0..=80.0),
        egui::Panel::bottom("octant_bottom_bar_expanded")
            .resizable(true)
            .default_size(42.0)
            .size_range(38.0..=80.0),
        |ui, is_expanded| {
            if is_expanded {
                ui.horizontal_centered(|ui| show_contents(app, ui));
            } else {
                request_expand = show_collapsed_strip(ui);
            }
        },
    );

    app.layout.show_bottom_bar = expanded || request_expand;
}

/// Thin collapsed strip; returns `true` when clicked or dragged up.
fn show_collapsed_strip(ui: &mut egui::Ui) -> bool {
    let full_rect = ui.available_rect_before_wrap();
    let resp = ui
        .interact(
            full_rect,
            ui.id().with("collapsed_bottom_bar_interact"),
            egui::Sense::click_and_drag(),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand);

    let text_color = if resp.hovered() {
        ui.visuals().strong_text_color()
    } else {
        ui.visuals().text_color()
    };
    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 200.0).max(0.0) * 0.5);
            ui.icon(Icon::Play, IconSize::Xs);
            ui.label(
                egui::RichText::new("Playback (click or drag to expand)")
                    .small()
                    .color(text_color),
            );
        });
    });

    resp.clicked() || (resp.dragged() && resp.drag_delta().y < -1.0)
}

/// Measure every item, collapse until they fit around the minimum slider
/// width, then draw left items, the stretched slider, and right items.
fn show_contents(app: &mut OctantApp, ui: &mut egui::Ui) {
    let tl = Timeline::cached(app, ui.ctx());
    let spacing = ui.spacing().item_spacing.x;

    let mut widths = [ItemWidths::default(); ITEM_COUNT];
    for item in LEFT_ITEMS.into_iter().chain(RIGHT_ITEMS) {
        widths[item as usize] = items::item_widths(item, app, &tl, ui);
    }
    let compact = layout::compute_compact(&widths, ui.available_width() - SLIDER_MIN_W - spacing);

    for item in LEFT_ITEMS {
        items::show_item(item, compact.get(item), app, &tl, ui);
    }

    let right_w: f32 = RIGHT_ITEMS
        .iter()
        .map(|&item| {
            let w = widths[item as usize];
            if compact.get(item) { w.compact } else { w.full }
        })
        .sum();
    show_slider(app, &tl, ui, right_w + spacing);

    for item in RIGHT_ITEMS {
        items::show_item(item, compact.get(item), app, &tl, ui);
    }
}

/// Timeline slider filling the row except `reserve_right`.
fn show_slider(app: &mut OctantApp, tl: &Timeline, ui: &mut egui::Ui, reserve_right: f32) {
    ui.spacing_mut().slider_width = (ui.available_width() - reserve_right).max(SLIDER_MIN_W);
    let mut step = app.playback.current_timestep;
    let slider = egui::Slider::new(&mut step, 0..=tl.last_step)
        .show_value(false)
        .trailing_fill(true);
    // Fixed id so badges hiding before the slider never break a drag.
    let changed = ui
        .push_id("bottom_bar_slider", |ui| ui.add(slider).changed())
        .inner;
    if changed {
        app.request_step_or_load(step);
    }
}
