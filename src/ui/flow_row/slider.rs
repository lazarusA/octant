//! Sliders whose label moves above the track when the row is short of room.

use super::measure::layout_key;
use super::{cached_widths, fits, text_width};
use egui::{Id, Response, Slider, TextStyle, Ui, UiBuilder};
use std::hash::Hash;

/// Width of a value box beside a slider track: the fixed boxes of the
/// dimension rows (7 digits), so their tracks end at the same x, and a
/// labeled slider's until its own is measured.
pub const VALUE_BOX_W: f32 = 56.0;
/// Narrowest slider track drawn when the row is short of room.
const MIN_TRACK_W: f32 = 40.0;

/// Width left for a slider track beside a `value_w` wide value box.
pub fn track_width(ui: &Ui, value_w: f32) -> f32 {
    (ui.available_width() - value_w - ui.spacing().item_spacing.x).max(MIN_TRACK_W)
}

/// Where a slider's label goes when the row has room for it.
#[derive(Clone, Copy)]
pub enum SliderLabel<'a> {
    /// Before the slider, like a form label.
    Before(&'a str),
    /// After the value, like egui's `Slider::text`.
    After(&'a str),
}

/// `slider` with its label on the same row when the row fits it at egui's
/// default slider width; otherwise the label on its own line above a slider
/// that fills the row, naming it. `salt` names the slider in its `ui`
/// (fixed, so it survives a language switch); the slider keeps one id in
/// either layout, so a drag or an edit carries on when the layout changes.
///
/// Its value box follows the value's text: it is measured when drawn and the
/// widest seen is kept (until the fonts, language, scale or style change),
/// so a slider moves under its label at most once, never back and forth
/// while dragged. The slider's response.
pub fn labeled_slider(
    ui: &mut Ui,
    salt: impl Hash + std::fmt::Debug,
    label: SliderLabel<'_>,
    slider: Slider<'_>,
) -> Response {
    let text = match label {
        SliderLabel::Before(text) | SliderLabel::After(text) => text,
    };
    let salt = ("labeled_slider", salt);
    let [label_w] = cached_widths(ui, &salt, text, |ui| {
        [text_width(ui, text, TextStyle::Body)]
    });
    let id = ui.id().with(&salt);
    let value = ValueBox::load(ui, id.with("value_w"));
    let spacing = ui.spacing().item_spacing.x;
    let slider_w = ui.spacing().slider_width;
    let slider_id = id.with("slider");
    if !fits(ui, &[label_w, slider_w, value.width]) {
        return stacked(ui, text, slider, slider_id, value);
    }
    // The response's width, less everything but the value box.
    let (response, others) = match label {
        SliderLabel::After(text) => (
            add_slider(ui, slider_id, slider.text(text)),
            slider_w + spacing * 2.0 + label_w,
        ),
        SliderLabel::Before(text) => {
            let response = ui
                .horizontal(|ui| {
                    ui.label(text);
                    add_slider(ui, slider_id, slider)
                })
                .inner;
            (response, slider_w + spacing)
        }
    };
    value.widen(ui, response.rect.width() - others);
    response
}

/// `text` on its own line above `slider`, whose track fills the row beside
/// its `value` box.
fn stacked(ui: &mut Ui, text: &str, slider: Slider<'_>, id: Id, value: ValueBox) -> Response {
    let label = ui.label(text);
    let track = track_width(ui, value.width);
    let response = ui
        .scope_builder(UiBuilder::new().id(id), |ui| {
            ui.spacing_mut().slider_width = track;
            ui.add(slider)
        })
        .inner;
    let spacing = ui.spacing().item_spacing.x;
    value.widen(ui, response.rect.width() - track - spacing);
    response.labelled_by(label.id)
}

/// `slider` under the fixed `id`, wherever the layout puts it.
fn add_slider(ui: &mut Ui, id: Id, slider: Slider<'_>) -> Response {
    ui.scope_builder(UiBuilder::new().id(id), |ui| ui.add(slider))
        .inner
}

/// A labeled slider's value box width, kept under `id` with the
/// `layout_key` it was measured at.
#[derive(Clone, Copy)]
struct ValueBox {
    id: Id,
    key: u64,
    width: f32,
}

impl ValueBox {
    /// The width kept under `id`, or `VALUE_BOX_W` when nothing is kept for
    /// the current layout.
    fn load(ui: &Ui, id: Id) -> Self {
        let key = layout_key(ui);
        let width = ui
            .data(|d| d.get_temp::<(u64, f32)>(id))
            .filter(|&(kept, _)| kept == key)
            .map_or(VALUE_BOX_W, |(_, width)| width);
        Self { id, key, width }
    }

    /// Keeps `measured` when it is wider than the width kept.
    fn widen(self, ui: &Ui, measured: f32) {
        if measured > self.width + 0.5 {
            ui.data_mut(|d| d.insert_temp(self.id, (self.key, measured)));
            ui.ctx().request_repaint();
        }
    }
}
