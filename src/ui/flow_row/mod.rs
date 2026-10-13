//! Rows that stack when their items don't fit the panel's width, so longer
//! translations move onto new lines instead of running past the panel.
//!
//! Items are measured with the same text layout egui draws them with, once
//! per row until its text, the language, the fonts or the UI scale change
//! (`cached_widths`). Rows of plain left-aligned items use
//! `ui.horizontal_wrapped` instead.
//!
//! Right-aligned items are added by index, item 0 rightmost; stacked, they
//! keep their reading order, item 0 at the bottom.

mod measure;
mod slider;
#[cfg(test)]
mod tests;

pub use measure::{button_width, cached_widths, checkbox_width, combo_width, text_width};
pub use slider::{SliderLabel, VALUE_BOX_W, labeled_slider, track_width};

use egui::{Align, Layout, Ui, vec2};

/// Width of `widths` laid side by side, `item_spacing.x` apart.
pub fn row_width(ui: &Ui, widths: &[f32]) -> f32 {
    let gaps = widths.len().saturating_sub(1) as f32;
    widths.iter().sum::<f32>() + gaps * ui.spacing().item_spacing.x
}

/// Whether items `widths` wide fit on one row of `ui`.
pub fn fits(ui: &Ui, widths: &[f32]) -> bool {
    row_width(ui, widths) <= ui.available_width()
}

/// Items `widths` wide, each added by `add(ui, index)`: right-aligned on one
/// row when they fit, else stacked one per line, right-aligned.
pub fn right_aligned(ui: &mut Ui, widths: &[f32], mut add: impl FnMut(&mut Ui, usize)) {
    let count = widths.len();
    if fits(ui, widths) {
        // One row tall: a bare right-to-left layout would take the panel's height.
        let row = vec2(ui.available_width(), ui.spacing().interact_size.y);
        let layout = Layout::right_to_left(Align::Center);
        ui.allocate_ui_with_layout(row, layout, |ui| {
            (0..count).for_each(|i| add(ui, i));
        });
    } else {
        ui.with_layout(Layout::top_down(Align::Max), |ui| {
            (0..count).rev().for_each(|i| add(ui, i));
        });
    }
}

/// `leading` on the left and the trailing items (`trailing_widths` wide,
/// added by `trailing(ui, index)`) right-aligned on the same row when all
/// fit beside `leading_width`; otherwise `leading` on its own (wrapping) line
/// above them.
pub fn split(
    ui: &mut Ui,
    leading_width: f32,
    trailing_widths: &[f32],
    leading: impl FnOnce(&mut Ui),
    mut trailing: impl FnMut(&mut Ui, usize),
) {
    let gap = ui.spacing().item_spacing.x;
    if leading_width + gap + row_width(ui, trailing_widths) <= ui.available_width() {
        ui.horizontal(|ui| {
            leading(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                (0..trailing_widths.len()).for_each(|i| trailing(ui, i));
            });
        });
    } else {
        ui.horizontal_wrapped(leading);
        right_aligned(ui, trailing_widths, trailing);
    }
}
