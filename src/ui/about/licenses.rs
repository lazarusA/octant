//! Embedded license documents (third-party dependencies and colormap data) and a
//! virtualized viewer that lays out only the visible lines.

use super::license_wrap::{self, LineKind, VisualLine};
use crate::utils::colormap::LICENSES_TEXT;
use egui::{Color32, FontId, Label, Pos2, Rect, RichText, Sense, vec2};
use std::sync::Arc;

/// Notices for every crate, font and C library compiled into Octant, generated
/// by `cargo about` (see `about.toml`).
pub const THIRD_PARTY_LICENSES: &str =
    include_str!("../../../assets/licenses/THIRD_PARTY_LICENSES.md");

const THIRD_PARTY_URL: &str =
    "https://github.com/lazarusA/octant/blob/main/assets/licenses/THIRD_PARTY_LICENSES.md";
const COLORMAP_LICENSES_URL: &str =
    "https://github.com/lazarusA/octant/blob/main/assets/colormaps/LICENSES.md";

const VIEWER_HEIGHT: f32 = 300.0;
const FONT_SIZE: f32 = 12.0;
/// Extra space between lines, split above and below each one.
const LINE_GAP: f32 = 2.0;
const PADDING: i8 = 8;

/// A document's visual lines for one wrap width, kept in egui temp memory and
/// rebuilt only when the width changes (lines borrow the `'static` text).
#[derive(Clone, Default)]
struct Wrapped {
    columns: usize,
    lines: Arc<Vec<VisualLine>>,
}

fn wrapped(
    ui: &egui::Ui,
    salt: &'static str,
    text: &'static str,
    columns: usize,
) -> Arc<Vec<VisualLine>> {
    let id = egui::Id::new(("about_license_wrapped", salt));
    let cached: Option<Wrapped> = ui.data(|d| d.get_temp(id));
    if let Some(w) = cached.filter(|w| w.columns == columns) {
        return w.lines;
    }
    let lines = Arc::new(license_wrap::wrap(text, columns));
    let entry = Wrapped {
        columns,
        lines: Arc::clone(&lines),
    };
    ui.data_mut(|d| d.insert_temp(id, entry));
    lines
}

/// Collapsible section with the third-party notices.
pub fn show_third_party_licenses(ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Third-party licenses")
        .id_salt(("about_third_party_licenses", 0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(
                    "Octant includes open-source crates, fonts and C libraries (netCDF, \
                     HDF5), each under its own license.",
                )
                .small()
                .weak(),
            );
            show_document(ui, "third_party", THIRD_PARTY_LICENSES, THIRD_PARTY_URL);
        });
}

/// The colormap `LICENSES.md`, shown verbatim (wrapped to the viewer width).
pub fn show_colormap_licenses(ui: &mut egui::Ui) {
    show_document(ui, "colormaps", LICENSES_TEXT, COLORMAP_LICENSES_URL);
}

/// Font metrics and colors of the document lines.
struct LineStyle {
    font: FontId,
    glyph_width: f32,
    row_height: f32,
    heading: Color32,
    dim: Color32,
}

/// Framed, vertically scrolling monospace view of `text`, wrapped to the
/// viewer width; only the visible lines are laid out.
pub(super) fn show_document(ui: &mut egui::Ui, salt: &'static str, text: &'static str, url: &str) {
    let font = FontId::monospace(FONT_SIZE);
    let (font_height, glyph_width) =
        ui.fonts_mut(|f| (f.row_height(&font), f.glyph_width(&font, ' ')));
    let visuals = ui.visuals();
    let style = LineStyle {
        font,
        glyph_width: glyph_width.max(1.0),
        row_height: font_height + LINE_GAP,
        heading: visuals.strong_text_color(),
        dim: visuals.weak_text_color(),
    };
    egui::Frame::new()
        .fill(visuals.extreme_bg_color)
        .stroke(visuals.widgets.noninteractive.bg_stroke)
        .corner_radius(4)
        .inner_margin(PADDING)
        .show(ui, |ui| {
            let width = ui.available_width() - ui.spacing().scroll.allocated_width();
            let columns = (width / style.glyph_width).floor().max(0.0) as usize;
            let lines = wrapped(ui, salt, text, columns);
            let content = vec2(ui.available_width(), style.row_height * lines.len() as f32);
            egui::ScrollArea::vertical()
                .id_salt(("about_license_document", salt))
                .max_height(VIEWER_HEIGHT)
                // Windows and areas offer only last frame's size; without this
                // the viewer could stay as short as it first appeared.
                .min_scrolled_height(VIEWER_HEIGHT)
                .auto_shrink([false, true])
                .show_viewport(ui, |ui, viewport| {
                    let (rect, _) = ui.allocate_exact_size(content, Sense::hover());
                    for row in visible_rows(viewport, style.row_height, lines.len()) {
                        if let Some(line) = lines.get(row) {
                            show_line(ui, rect.min, row, line, &style);
                        }
                    }
                });
        });
    ui.hyperlink_to("View on GitHub", url);
}

/// Draws visual line `row` as a selectable label starting at its indent.
fn show_line(ui: &mut egui::Ui, origin: Pos2, row: usize, line: &VisualLine, style: &LineStyle) {
    let color = match line.kind {
        LineKind::Body => Color32::PLACEHOLDER,
        LineKind::Heading => style.heading,
        LineKind::Fence => style.dim,
    };
    // Laid out every frame (an egui cache hit), never kept across frames:
    // theme switches rebuild the glyph atlas.
    let galley = ui
        .painter()
        .layout_no_wrap(line.text.to_owned(), style.font.clone(), color);
    let min = origin
        + vec2(
            f32::from(line.indent) * style.glyph_width,
            row as f32 * style.row_height + LINE_GAP / 2.0,
        );
    // `put` centers its widget, so the rect is exactly the galley: the line
    // starts at its indent instead of the middle of the row.
    ui.put(Rect::from_min_size(min, galley.size()), Label::new(galley));
}

/// Rows of height `row_height` that intersect `viewport` (content coordinates).
fn visible_rows(viewport: Rect, row_height: f32, len: usize) -> std::ops::Range<usize> {
    if row_height <= 0.0 {
        return 0..0;
    }
    let first = (viewport.min.y / row_height).floor().max(0.0) as usize;
    let last = (viewport.max.y / row_height).ceil().max(0.0) as usize;
    first.min(len)..last.min(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn third_party_notices_cover_bundled_components() {
        for component in [
            "epaint_default_fonts",
            "SIL Open Font License 1.1",
            "Ubuntu Font Licence",
            "Source Foundry",
            "netcdf-src",
            "hdf5-metno-src",
            "UC LLNL",
            "Unicode License v3",
        ] {
            assert!(
                THIRD_PARTY_LICENSES.contains(component),
                "missing {component}"
            );
        }
        assert!(THIRD_PARTY_LICENSES.lines().count() > 1000);
        assert!(!LICENSES_TEXT.is_empty());
    }

    #[test]
    fn scrolled_to_the_bottom_shows_the_last_line() {
        let (row_height, len) = (14.0, 1000);
        let bottom = row_height * len as f32;
        let viewport =
            Rect::from_min_max(egui::pos2(0.0, bottom - 260.0), egui::pos2(300.0, bottom));
        let rows = visible_rows(viewport, row_height, len);
        assert_eq!(rows.end, len);
        assert_eq!(rows.start, len - (260.0_f32 / row_height).ceil() as usize);
        let top = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(300.0, 260.0));
        assert_eq!(visible_rows(top, row_height, len), 0..19);
        assert_eq!(visible_rows(top, 0.0, len), 0..0);
    }
}
