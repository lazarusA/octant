//! Rows that stack when their items don't fit the panel's width, so longer
//! translations move onto new lines instead of running past the panel.
//!
//! Items are measured with the same text layout egui draws them with (its
//! galley cache makes an unchanged label a hash lookup). Rows of plain
//! left-aligned items use `ui.horizontal_wrapped` instead.

use egui::{Align, Layout, Response, Slider, TextStyle, TextWrapMode, Ui, WidgetText, vec2};

/// Room reserved for a slider's value box, as in the dimension sliders.
const VALUE_W: f32 = crate::ui::variables_panel::dimension_slider::double_slider::VALUE_BOX_W;
/// Narrowest slider track drawn when the row is short of room.
const MIN_TRACK_W: f32 = 40.0;

/// Width of `text` on one line, in `style` unless the text sets its own.
pub fn text_width(ui: &Ui, text: impl Into<WidgetText>, style: TextStyle) -> f32 {
    let text: WidgetText = text.into();
    if text.is_empty() {
        return 0.0;
    }
    text.into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, style)
        .size()
        .x
}

/// Width of a text button, selectable label, toggle or menu button labeled
/// `text`.
pub fn button_width(ui: &Ui, text: impl Into<WidgetText>) -> f32 {
    text_width(ui, text, TextStyle::Button) + ui.spacing().button_padding.x * 2.0
}

/// Width of a `ComboBox` showing `selected` at egui's default width.
pub fn combo_width(ui: &Ui, selected: impl Into<WidgetText>) -> f32 {
    let spacing = ui.spacing();
    let content =
        text_width(ui, selected, TextStyle::Button) + spacing.icon_spacing + spacing.icon_width;
    content.max(spacing.combo_width - spacing.button_padding.x * 2.0)
        + spacing.button_padding.x * 2.0
}

/// Width of `widths` laid side by side, `item_spacing.x` apart.
pub fn row_width(ui: &Ui, widths: &[f32]) -> f32 {
    let gaps = widths.len().saturating_sub(1) as f32;
    widths.iter().sum::<f32>() + gaps * ui.spacing().item_spacing.x
}

/// Whether items `widths` wide fit on one row of `ui`.
pub fn fits(ui: &Ui, widths: &[f32]) -> bool {
    row_width(ui, widths) <= ui.available_width()
}

/// The items `add` places, `widths` wide, right-aligned on one row (the first
/// added rightmost) when they fit, else stacked one per line, right-aligned
/// (the first added on top).
pub fn right_aligned<R>(ui: &mut Ui, widths: &[f32], add: impl FnOnce(&mut Ui) -> R) -> R {
    if fits(ui, widths) {
        // One row tall: a bare right-to-left layout would take the panel's height.
        let row = vec2(ui.available_width(), ui.spacing().interact_size.y);
        let layout = Layout::right_to_left(Align::Center);
        ui.allocate_ui_with_layout(row, layout, add).inner
    } else {
        ui.with_layout(Layout::top_down(Align::Max), add).inner
    }
}

/// `leading` on the left and `trailing` (items `trailing_widths` wide)
/// right-aligned on the same row when all fit beside `leading_width`;
/// otherwise `leading` on its own (wrapping) line above `trailing`.
pub fn split<R>(
    ui: &mut Ui,
    leading_width: f32,
    trailing_widths: &[f32],
    leading: impl FnOnce(&mut Ui),
    trailing: impl FnOnce(&mut Ui) -> R,
) -> R {
    let trailing_width = row_width(ui, trailing_widths);
    let gap = ui.spacing().item_spacing.x;
    if leading_width + gap + trailing_width <= ui.available_width() {
        ui.horizontal(|ui| {
            leading(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), trailing)
                .inner
        })
        .inner
    } else {
        ui.horizontal_wrapped(leading);
        right_aligned(ui, trailing_widths, trailing)
    }
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
/// that fills the row. The slider's response.
pub fn labeled_slider(ui: &mut Ui, label: SliderLabel<'_>, slider: Slider<'_>) -> Response {
    let text = match label {
        SliderLabel::Before(text) | SliderLabel::After(text) => text,
    };
    let label_w = text_width(ui, text, TextStyle::Body);
    let slider_w = ui.spacing().slider_width;
    if fits(ui, &[label_w, slider_w, VALUE_W]) {
        return match label {
            SliderLabel::After(text) => ui.add(slider.text(text)),
            SliderLabel::Before(text) => {
                ui.horizontal(|ui| {
                    ui.label(text);
                    ui.add(slider)
                })
                .inner
            }
        };
    }
    ui.label(text);
    ui.scope(|ui| {
        let spacing = ui.spacing().item_spacing.x;
        ui.spacing_mut().slider_width = (ui.available_width() - VALUE_W - spacing).max(MIN_TRACK_W);
        ui.add(slider)
    })
    .inner
}
