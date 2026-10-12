//! Widths of the items a flow row lays out, without allocating them, and a
//! per-row cache so they are measured only when they can change.

use crate::ui::temp_cache::{cached, hash_key};
use egui::{Id, TextStyle, TextWrapMode, Ui, WidgetText};
use std::hash::Hash;

/// Width of `text` on one line in `style`.
pub fn text_width(ui: &Ui, text: &str, style: TextStyle) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    WidgetText::from(text)
        .into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, style)
        .size()
        .x
}

/// Width of a text button, selectable label, toggle or menu button labeled
/// `text` in `style` (`TextStyle::Button` unless the label sets its own).
pub fn button_width(ui: &Ui, text: &str, style: TextStyle) -> f32 {
    text_width(ui, text, style) + ui.spacing().button_padding.x * 2.0
}

/// Width of a `ComboBox` showing `selected` at egui's default width.
pub fn combo_width(ui: &Ui, selected: &str) -> f32 {
    let spacing = ui.spacing();
    let pad = spacing.button_padding.x * 2.0;
    let content =
        text_width(ui, selected, TextStyle::Button) + spacing.icon_spacing + spacing.icon_width;
    content.max(spacing.combo_width - pad) + pad
}

/// Width of a checkbox without a label: egui makes it at least one row
/// (`interact_size.y`) square.
pub fn checkbox_width(ui: &Ui) -> f32 {
    let spacing = ui.spacing();
    spacing.icon_width.max(spacing.interact_size.y)
}

/// The widths `measure` returns for the row `salt`, kept in egui temp memory
/// and measured again only when `key` (the row's text), the language, the
/// fonts or the UI scale change.
pub fn cached_widths<const N: usize>(
    ui: &Ui,
    salt: impl Hash + std::fmt::Debug,
    key: impl Hash,
    measure: impl FnOnce(&Ui) -> [f32; N],
) -> [f32; N] {
    let ctx = ui.ctx();
    let key = hash_key((
        crate::ui::fonts::generation(ctx),
        crate::i18n::language(),
        ui.pixels_per_point().to_bits(),
        key,
    ));
    cached(ctx, Id::new("flow_row_widths").with(salt), key, || {
        measure(ui)
    })
}
