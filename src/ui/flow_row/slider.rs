//! Sliders whose label moves above the track when the row is short of room.

use super::{cached_widths, fits, text_width};
use egui::{Id, Response, Slider, TextStyle, Ui};

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
/// that fills the row. Its value box (which follows the value's text) is
/// measured each time it is drawn and kept for the next frame. The slider's
/// response.
pub fn labeled_slider(ui: &mut Ui, label: SliderLabel<'_>, slider: Slider<'_>) -> Response {
    let text = match label {
        SliderLabel::Before(text) | SliderLabel::After(text) => text,
    };
    let id = ui.id().with(("labeled_slider", text));
    let [label_w] = cached_widths(ui, id, text, |ui| [text_width(ui, text, TextStyle::Body)]);
    let value_id = id.with("value_w");
    let value_w = ui
        .data(|d| d.get_temp::<f32>(value_id))
        .unwrap_or(VALUE_BOX_W);
    let spacing = ui.spacing().item_spacing.x;
    let slider_w = ui.spacing().slider_width;
    // The response's width, less everything but the value box.
    let (response, others) = if fits(ui, &[label_w, slider_w, value_w]) {
        match label {
            SliderLabel::After(text) => (
                ui.add(slider.text(text)),
                slider_w + spacing * 2.0 + label_w,
            ),
            SliderLabel::Before(text) => {
                let response = ui
                    .horizontal(|ui| {
                        ui.label(text);
                        ui.add(slider)
                    })
                    .inner;
                (response, slider_w + spacing)
            }
        }
    } else {
        ui.label(text);
        ui.scope(|ui| {
            let track = track_width(ui, value_w);
            ui.spacing_mut().slider_width = track;
            (ui.add(slider), track + spacing)
        })
        .inner
    };
    remember_value_width(ui, value_id, value_w, response.rect.width() - others);
    response
}

/// Keeps `measured` under `id` when it differs from the `kept` width.
fn remember_value_width(ui: &Ui, id: Id, kept: f32, measured: f32) {
    if measured > 0.0 && (measured - kept).abs() > 0.5 {
        ui.data_mut(|d| d.insert_temp(id, measured));
        ui.ctx().request_repaint();
    }
}
