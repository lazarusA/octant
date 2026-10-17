//! egui's number widgets in the UI language. `install` makes every
//! `DragValue` and `Slider` show its number with the language's decimal
//! separator (egui formats them through the style's `number_formatter`, read
//! on each call, so a language switch needs no reinstall); `.localized()`
//! gives one a parser that accepts what that display shows, a comma or a
//! point. Every `DragValue` and `Slider` takes `.localized()` (checked by
//! `i18n::guard_tests`).

use crate::i18n::number;

/// Installs the localized number formatter in the dark and light styles.
pub fn install(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style.number_formatter = egui::style::NumberFormatter::new(|value, decimals| {
            number::localize_string(egui::emath::format_with_decimals_in_range(value, decimals))
        });
    });
}

/// Number widgets that read typed numbers in the UI language.
pub trait LocalizedInput {
    /// Parses typed numbers with [`number::parse`].
    fn localized(self) -> Self;
}

impl LocalizedInput for egui::DragValue<'_> {
    fn localized(self) -> Self {
        self.custom_parser(number::parse)
    }
}

impl LocalizedInput for egui::Slider<'_> {
    fn localized(self) -> Self {
        self.custom_parser(number::parse)
    }
}

impl LocalizedInput for egui::RangeSlider<'_> {
    fn localized(self) -> Self {
        self.custom_parser(number::parse)
    }
}
