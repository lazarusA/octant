//! Hero header title, status pill, idle hints, drag cues, and warning banners.

use super::style::{BODY_FONT, SMALL_FONT, content_width, fit_text, title_font};
use crate::ui::icons::{ICON_GAP, Icon, IconSize, IconTone};

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
    icon: Icon,
    icon_color: egui::Color32,
    text: &str,
    text_color: egui::Color32,
) {
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

/// Helper caption under the intake bar, centered in the hero column.
pub fn render_idle_hint(ui: &mut egui::Ui) {
    let text = fit_text(
        ui,
        &[
            "paste URL, local path, or drag & drop files anywhere",
            "paste URL, path, or drag & drop files",
            "paste URL or drop files",
        ],
        SMALL_FONT,
        content_width(ui.available_width()),
    );
    ui.label(
        egui::RichText::new(text)
            .monospace()
            .size(SMALL_FONT)
            .color(ui.visuals().strong_text_color()),
    );
}

pub fn render_drag_hover_cue(ui: &mut egui::Ui) {
    render_banner(
        ui,
        Icon::DropTray,
        IconTone::Accent,
        &[
            "Drop dataset to load (.nc, .h5, .zarr, .icechunk, .tif)",
            "Drop dataset (.nc, .zarr, .icechunk, .tif)",
            "Drop dataset to load",
        ],
    );
}

pub fn render_warning_banner(ui: &mut egui::Ui) {
    render_banner(
        ui,
        Icon::Warning,
        IconTone::Warning,
        &[
            "Unsupported type. Supported: .nc, .h5, .zarr, .icechunk, .tif",
            "Unsupported format (.nc, .zarr, .icechunk, .tif)",
            "Unsupported format",
        ],
    );
}

/// Content-width banner tinted with `tone`: icon at the left, then the
/// longest of `messages` that fits.
fn render_banner(ui: &mut egui::Ui, icon: Icon, tone: IconTone, messages: &[&str]) {
    const PAD_X: f32 = 12.0;
    let icon_px = IconSize::Sm.px();

    let width = content_width(ui.available_width());
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, BANNER_HEIGHT), egui::Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }

    let visuals = ui.visuals();
    let color = tone.color(visuals);
    let fill = tone.themed_tint(visuals, 24, 16);
    ui.painter().rect(
        rect,
        6.0,
        fill,
        egui::Stroke::new(1.0, color),
        egui::StrokeKind::Inside,
    );

    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + PAD_X, rect.center().y - icon_px * 0.5),
        egui::Vec2::splat(icon_px),
    );
    icon.paint(ui.painter(), icon_rect, color, visuals.dark_mode);

    let text_x = icon_rect.right() + ICON_GAP + 2.0;
    let msg = fit_text(ui, messages, BODY_FONT, rect.right() - PAD_X - text_x);
    ui.painter().text(
        egui::pos2(text_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        msg,
        egui::FontId::monospace(BODY_FONT),
        color,
    );
}
