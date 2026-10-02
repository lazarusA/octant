//! Hero header title, status pill, idle hints, drag cues, and warning banners.

use super::intake::INTAKE_TEXT_INSET;
use super::style::{BODY_FONT, SMALL_FONT, content_width, fit_text, title_font};

/// Height of the drag-hover and warning banners.
const BANNER_HEIGHT: f32 = 40.0;

/// "OCTANT" wordmark under the cube: spaced monospace capitals in the strong
/// text color.
pub fn header_title(ui: &mut egui::Ui) {
    let font_size = title_font(ui.available_width());
    let mut job = egui::text::LayoutJob::default();
    job.append(
        "OCTANT",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(font_size),
            color: ui.visuals().strong_text_color(),
            extra_letter_spacing: font_size * 0.35,
            ..Default::default()
        },
    );
    ui.label(job);
}

/// Centered status line: icon followed by `text`, which wraps onto at most
/// two centered lines within the hero content width.
pub fn render_status_pill(
    ui: &mut egui::Ui,
    icon: crate::ui::icons::Icon,
    icon_color: egui::Color32,
    text: &str,
    text_color: egui::Color32,
) {
    use crate::ui::icons::{ICON_GAP, IconSize};

    let icon_px = IconSize::Xs.px();
    let max_text_w = content_width(ui.available_width()) - icon_px - ICON_GAP;

    let mut job = egui::text::LayoutJob::simple(
        text.to_string(),
        egui::FontId::monospace(BODY_FONT),
        text_color,
        max_text_w,
    );
    job.wrap.max_rows = 2;
    job.halign = egui::Align::Center;
    let galley = ui.painter().layout_job(job);

    // Allocating the exact block size lets the parent vertical_centered
    // layout center it horizontally.
    let text_w = galley.size().x;
    let size = egui::vec2(icon_px + ICON_GAP + text_w, galley.size().y.max(icon_px));
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }

    let first_row_h = galley.rows.first().map_or(icon_px, |r| r.height());
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x, rect.min.y + (first_row_h - icon_px) * 0.5),
        egui::vec2(icon_px, icon_px),
    );
    icon.paint(ui.painter(), icon_rect, icon_color, ui.visuals().dark_mode);

    // Offset by the galley's own bounds: center-aligned rows may start at a
    // negative x relative to the galley origin.
    let text_min = egui::pos2(rect.min.x + icon_px + ICON_GAP, rect.min.y);
    let text_pos = text_min - galley.rect.min.to_vec2();
    ui.painter().galley(text_pos, galley, text_color);
}

/// Helper caption under the intake bar, left-aligned with the bar's edge.
pub fn render_idle_hint(ui: &mut egui::Ui) {
    let width = content_width(ui.available_width());
    // Inset to line up with the text inside the intake frame.
    let inset = INTAKE_TEXT_INSET;
    let text = fit_text(
        ui,
        &[
            "paste URL, local path, or drag & drop files anywhere",
            "paste URL, path, or drag & drop files",
            "paste URL or drop files",
        ],
        SMALL_FONT,
        width - inset,
    );
    let color = ui.visuals().strong_text_color();
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), egui::FontId::monospace(SMALL_FONT), color);

    // A full content-width row is centered by the parent column; the text is
    // then painted from its left edge.
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(width, galley.size().y), egui::Sense::hover());
    ui.painter()
        .galley(egui::pos2(rect.min.x + inset, rect.min.y), galley, color);
}

pub fn render_drag_hover_cue(ui: &mut egui::Ui) {
    let width = content_width(ui.available_width());
    let height = BANNER_HEIGHT;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let is_dark = ui.visuals().dark_mode;
        let accent = crate::ui::icons::IconTone::Accent.color(ui.visuals());
        let bg =
            crate::ui::icons::IconTone::Accent.tint(ui.visuals(), if is_dark { 22 } else { 16 });

        ui.painter().rect(
            rect,
            6.0,
            bg,
            egui::Stroke::new(1.2, accent),
            egui::StrokeKind::Inside,
        );

        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 20.0, rect.center().y),
            egui::vec2(
                crate::ui::icons::IconSize::Sm.px(),
                crate::ui::icons::IconSize::Sm.px(),
            ),
        );
        crate::ui::icons::Icon::DropTray.paint(ui.painter(), icon_rect, accent, is_dark);

        let msg = fit_text(
            ui,
            &[
                "Drop dataset to load (.nc, .h5, .zarr, .icechunk, .tif)",
                "Drop dataset (.nc, .zarr, .icechunk, .tif)",
                "Drop dataset to load",
            ],
            BODY_FONT,
            width - 44.0,
        );

        ui.painter().text(
            egui::pos2(rect.left() + 34.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            msg,
            egui::FontId::monospace(BODY_FONT),
            accent,
        );
    }
}

pub fn render_warning_banner(ui: &mut egui::Ui) {
    let width = content_width(ui.available_width());
    let height = BANNER_HEIGHT;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let is_dark = ui.visuals().dark_mode;
        let warning_color = crate::ui::icons::IconTone::Warning.color(ui.visuals());
        let bg =
            crate::ui::icons::IconTone::Warning.tint(ui.visuals(), if is_dark { 26 } else { 18 });

        ui.painter().rect(
            rect,
            6.0,
            bg,
            egui::Stroke::new(1.0, warning_color),
            egui::StrokeKind::Inside,
        );

        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 18.0, rect.center().y),
            egui::vec2(
                crate::ui::icons::IconSize::Sm.px(),
                crate::ui::icons::IconSize::Sm.px(),
            ),
        );
        crate::ui::icons::Icon::Warning.paint(ui.painter(), icon_rect, warning_color, is_dark);

        let msg = fit_text(
            ui,
            &[
                "Unsupported type. Supported: .nc, .h5, .zarr, .icechunk, .tif",
                "Unsupported format (.nc, .zarr, .icechunk, .tif)",
                "Unsupported format",
            ],
            BODY_FONT,
            width - 42.0,
        );

        ui.painter().text(
            egui::pos2(rect.left() + 32.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            msg,
            egui::FontId::monospace(BODY_FONT),
            warning_color,
        );
    }
}
