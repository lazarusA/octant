//! Size and tone reference strip for the About dialog's icon gallery.
//!
//! Every entry is a single fixed-size item, so both rows wrap to the dialog
//! width instead of widening it.

use crate::ui::icons::{ICON_GAP, Icon, IconSize, IconTone};
use egui::{Align2, Color32, FontFamily, FontId, Rect, Sense, TextStyle, Ui, pos2, vec2};

/// Icons shown per size: one stroke-heavy, one filled, one detailed, so
/// weight differences across sizes are easy to compare.
const SAMPLE_ICONS: [Icon; 3] = [Icon::Settings, Icon::Play, Icon::Dataset];
/// Horizontal gap between wrapped items.
const ITEM_GAP: f32 = 14.0;
/// Gap between a size card's label and its icons.
const LABEL_GAP: f32 = 4.0;

/// Collapsible reference showing every [`IconSize`] and [`IconTone`].
pub fn show_scale_reference(ui: &mut Ui) {
    egui::CollapsingHeader::new("Sizes & tones")
        .id_salt(("about_icons", "scale_reference"))
        .default_open(false)
        .show(ui, scale_rows);
}

/// Size cards and tone chips, each wrapped to the current width.
pub(super) fn scale_rows(ui: &mut Ui) {
    ui.set_max_width(ui.available_width());
    ui.spacing_mut().item_spacing = vec2(ITEM_GAP, 8.0);
    ui.horizontal_wrapped(|ui| {
        for size in IconSize::ALL {
            size_card(ui, size);
        }
    });
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        for tone in IconTone::ALL {
            tone_chip(ui, tone);
        }
    });
}

fn small_font(ui: &Ui, family: FontFamily) -> FontId {
    FontId::new(TextStyle::Small.resolve(ui.style()).size, family)
}

/// Size label above the sample icons at that size. Cards share the `Lg`
/// height so the row stays aligned when it wraps.
fn size_card(ui: &mut Ui, size: IconSize) {
    let weak = ui.visuals().weak_text_color();
    let galley = ui.painter().layout_no_wrap(
        size_label(size).to_owned(),
        small_font(ui, FontFamily::Monospace),
        weak,
    );
    let px = size.px();
    let row_h = IconSize::Lg.px();
    let n = SAMPLE_ICONS.len() as f32;
    let icons_w = n * px + (n - 1.0) * ICON_GAP;
    let text = galley.size();
    let desired = vec2(text.x.max(icons_w), text.y + LABEL_GAP + row_h);
    let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    ui.painter().galley(rect.min, galley, weak);
    let y = rect.min.y + text.y + LABEL_GAP + (row_h - px) * 0.5;
    let (color, dark) = (ui.visuals().text_color(), ui.visuals().dark_mode);
    for (i, icon) in SAMPLE_ICONS.iter().enumerate() {
        let x = rect.min.x + i as f32 * (px + ICON_GAP);
        let icon_rect = Rect::from_min_size(pos2(x, y), vec2(px, px));
        icon.paint(ui.painter(), icon_rect, color, dark);
    }
}

/// Bolt icon and tone name, both in the tone's color.
fn tone_chip(ui: &mut Ui, tone: IconTone) {
    let color = tone.color(ui.visuals());
    let font = small_font(ui, FontFamily::Proportional);
    let galley = ui
        .painter()
        .layout_no_wrap(tone.name().to_owned(), font, Color32::PLACEHOLDER);
    let px = IconSize::Sm.px();
    let text = galley.size();
    let desired = vec2(px + ICON_GAP + text.x, px.max(text.y));
    let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let icon_rect = Rect::from_min_size(pos2(rect.min.x, rect.center().y - px * 0.5), vec2(px, px));
    Icon::Bolt.paint(ui.painter(), icon_rect, color, ui.visuals().dark_mode);
    let text_pos = Align2::LEFT_CENTER
        .align_size_within_rect(text, rect.with_min_x(icon_rect.max.x + ICON_GAP))
        .min;
    ui.painter().galley(text_pos, galley, color);
}

fn size_label(size: IconSize) -> &'static str {
    match size {
        IconSize::Xs => "Xs 12",
        IconSize::Sm => "Sm 14",
        IconSize::Md => "Md 18",
        IconSize::Lg => "Lg 24",
    }
}

#[cfg(test)]
mod tests {
    use super::scale_rows;

    /// The reference rows wrap inside narrow dialogs instead of widening them.
    #[test]
    fn scale_rows_never_exceed_available_width() {
        let ctx = egui::Context::default();
        for width in [160.0_f32, 240.0, 420.0] {
            let mut used = 0.0;
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui.allocate_ui(egui::vec2(width, 600.0), |ui| {
                    ui.set_max_width(width);
                    scale_rows(ui);
                    used = ui.min_rect().width();
                });
            });
            out.textures_delta.clear();
            assert!(used <= width + 0.5, "rows used {used} of {width}");
        }
    }
}
