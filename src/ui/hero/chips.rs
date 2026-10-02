//! Sample slash chips and interactive quick-load chip buttons.

use super::style::{BODY_FONT, GUTTER, SMALL_FONT, gap};
use crate::app::OctantApp;

const SAMPLES: [(&str, &str, &str); 3] = [
    (
        "/seasfire",
        "https://s3.bgc-jena.mpg.de:9000/misc/seasfire_rechunked.zarr",
        "Global wildfire & climate rechunked dataset (Zarr)",
    ),
    (
        "/sentinel-2",
        "https://sentinel-cogs.s3.us-west-2.amazonaws.com/sentinel-s2-l2a-cogs/36/Q/WD/2020/7/S2A_36QWD_20200701_0_L2A/TCI.tif",
        "Sentinel-2 L2A True Color COG (AWS S3)",
    ),
    (
        "/procedural-4d",
        "procedural://volume4d",
        "Synthetic 4D spatiotemporal volume",
    ),
];
const COUNT: usize = SAMPLES.len();
const PREFIX: &str = "try:";

/// Inner padding of a chip around its label (horizontal, vertical).
const CHIP_PADDING: egui::Vec2 = egui::vec2(14.0, 7.0);
/// Horizontal space between chips.
const CHIP_GAP: f32 = 8.0;

/// One row of the chip layout: the item range it holds and its width.
#[derive(Clone, Copy, Default)]
struct Line {
    start: usize,
    end: usize,
    width: f32,
    height: f32,
}

/// Centered rows of sample chips, preceded by a `try:` prefix. Rows wrap as
/// whole chips and every row stays centered.
pub fn sample_slash_chips_row(ui: &mut egui::Ui, app: &mut OctantApp) {
    let prefix_w = ui
        .painter()
        .layout_no_wrap(
            PREFIX.to_string(),
            egui::FontId::monospace(SMALL_FONT),
            ui.visuals().weak_text_color(),
        )
        .size()
        .x;

    // Item 0 is the prefix, items 1..=COUNT are the chips.
    let mut widths = [0.0_f32; COUNT + 1];
    widths[0] = prefix_w;
    for (w, (label, _, _)) in widths[1..].iter_mut().zip(SAMPLES) {
        *w = chip_galley(ui, label, false).size().x + CHIP_PADDING.x * 2.0;
    }

    let max_w = (ui.available_width() - 2.0 * GUTTER).max(0.0);
    let (mut lines, line_count) = pack_lines(&widths, max_w);
    let chip_height = chip_galley(ui, SAMPLES[0].0, false).size().y + CHIP_PADDING.y * 2.0;
    for line in &mut lines[..line_count] {
        line.height = chip_height;
    }

    for (n, line) in lines[..line_count].iter().enumerate() {
        if n > 0 {
            ui.add_space(gap::CHIP_ROWS);
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = CHIP_GAP;
            ui.add_space(((ui.available_width() - line.width) * 0.5).max(0.0));
            for item in line.start..line.end {
                if item == 0 {
                    render_prefix(ui, line.height);
                    continue;
                }
                let (label, uri, desc) = SAMPLES[item - 1];
                if render_ghost_slash_chip(ui, label, desc).clicked() {
                    app.hero_state.input = uri.to_string();
                    app.submit_or_activate_source(uri, None);
                }
            }
        });
    }
}

/// Greedily pack items into rows no wider than `max_w`. Every row holds at
/// least one item, so an over-wide chip gets a row of its own.
fn pack_lines(widths: &[f32; COUNT + 1], max_w: f32) -> ([Line; COUNT + 1], usize) {
    let mut lines = [Line::default(); COUNT + 1];
    let mut count = 0;
    let mut current = Line::default();
    for (i, &w) in widths.iter().enumerate() {
        let needed = if current.end > current.start {
            current.width + CHIP_GAP + w
        } else {
            w
        };
        if needed > max_w && current.end > current.start {
            lines[count] = current;
            count += 1;
            current = Line {
                start: i,
                end: i + 1,
                width: w,
                height: 0.0,
            };
        } else {
            current.end = i + 1;
            current.width = needed;
        }
    }
    lines[count] = current;
    (lines, count + 1)
}

/// The `try:` prefix, vertically centered in a box as tall as a chip so it
/// lines up with the chips beside it.
fn render_prefix(ui: &mut egui::Ui, height: f32) {
    let galley = ui.painter().layout_no_wrap(
        PREFIX.to_string(),
        egui::FontId::monospace(SMALL_FONT),
        ui.visuals().weak_text_color(),
    );
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(galley.size().x, height), egui::Sense::hover());
    let pos = egui::pos2(rect.min.x, rect.center().y - galley.size().y * 0.5);
    ui.painter()
        .galley(pos, galley, ui.visuals().weak_text_color());
}

/// Label galley with a dimmed leading slash.
fn chip_galley(ui: &egui::Ui, label: &str, hovered: bool) -> std::sync::Arc<egui::Galley> {
    let font_id = egui::FontId::monospace(BODY_FONT);
    let text_color = if hovered {
        ui.visuals().strong_text_color()
    } else {
        ui.visuals().text_color()
    };
    let format = |color| egui::TextFormat {
        font_id: font_id.clone(),
        color,
        ..Default::default()
    };

    let mut job = egui::text::LayoutJob::default();
    match label.strip_prefix('/') {
        Some(rest) => {
            job.append("/", 0.0, format(ui.visuals().weak_text_color()));
            job.append(rest, 0.0, format(text_color));
        }
        None => job.append(label, 0.0, format(text_color)),
    }
    ui.painter().layout_job(job)
}

/// Pill-shaped sample chip: plain text at rest, filled on hover.
pub fn render_ghost_slash_chip(ui: &mut egui::Ui, label: &str, desc: &str) -> egui::Response {
    let rest_galley = chip_galley(ui, label, false);
    let desired_size = rest_galley.size() + CHIP_PADDING * 2.0;
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        let widgets = &ui.visuals().widgets;
        // Borderless: only the hover fill marks the pill.
        if hovered {
            ui.painter()
                .rect_filled(rect, rect.height() * 0.5, widgets.hovered.bg_fill);
        }

        let galley = if hovered {
            chip_galley(ui, label, true)
        } else {
            rest_galley
        };
        let text_pos = rect.center() - galley.size() * 0.5;
        ui.painter()
            .galley(text_pos, galley, ui.visuals().text_color());
    }

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(format!("Load sample: {desc}"))
}

#[cfg(test)]
mod tests {
    use super::{COUNT, pack_lines};

    #[test]
    fn test_all_items_on_one_line_when_wide() {
        let (lines, n) = pack_lines(&[30.0, 100.0, 100.0, 100.0], 1000.0);
        assert_eq!(n, 1);
        assert_eq!((lines[0].start, lines[0].end), (0, COUNT + 1));
    }

    #[test]
    fn test_wraps_whole_items_and_covers_all() {
        let (lines, n) = pack_lines(&[30.0, 100.0, 100.0, 100.0], 250.0);
        assert!(n > 1);
        let mut next = 0;
        for line in &lines[..n] {
            assert_eq!(line.start, next);
            assert!(line.end > line.start);
            assert!(line.width <= 250.0);
            next = line.end;
        }
        assert_eq!(next, COUNT + 1);
    }

    #[test]
    fn test_over_wide_item_gets_its_own_line() {
        let (lines, n) = pack_lines(&[30.0, 400.0, 50.0, 50.0], 200.0);
        assert!(
            lines[..n]
                .iter()
                .any(|l| l.end - l.start == 1 && l.width == 400.0)
        );
    }
}
