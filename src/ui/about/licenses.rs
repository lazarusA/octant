//! Embedded license documents (third-party dependencies and colormap data) and a
//! virtualized viewer that lays out only the visible lines.

use crate::utils::colormap::LICENSES_TEXT;
use egui::{Color32, FontId, Galley, Label, Rect, RichText, Sense, vec2};
use std::ops::Range;
use std::sync::{Arc, LazyLock};

/// Notices for every crate, font and C library compiled into Octant, generated
/// by `cargo about` (see `about.toml`).
pub const THIRD_PARTY_LICENSES: &str =
    include_str!("../../../assets/licenses/THIRD_PARTY_LICENSES.md");

const THIRD_PARTY_URL: &str =
    "https://github.com/lazarusA/octant/blob/main/assets/licenses/THIRD_PARTY_LICENSES.md";
const COLORMAP_LICENSES_URL: &str =
    "https://github.com/lazarusA/octant/blob/main/assets/colormaps/LICENSES.md";

static THIRD_PARTY_LINES: LazyLock<Document> =
    LazyLock::new(|| Document::new(THIRD_PARTY_LICENSES));
static COLORMAP_LINES: LazyLock<Document> = LazyLock::new(|| Document::new(LICENSES_TEXT));

const VIEWER_HEIGHT: f32 = 260.0;
const FONT_SIZE: f32 = 10.5;
/// Columns a tab advances in the monospace font.
const TAB_COLUMNS: usize = 4;

/// Lines of an embedded document and its widest line in monospace columns, so
/// the scroll width is fixed instead of following the lines on screen.
struct Document {
    lines: Vec<&'static str>,
    columns: usize,
}

impl Document {
    fn new(text: &'static str) -> Self {
        let lines: Vec<&'static str> = text.lines().collect();
        let columns = lines
            .iter()
            .map(|l| {
                l.chars()
                    .map(|c| if c == '\t' { TAB_COLUMNS } else { 1 })
                    .sum()
            })
            .max()
            .unwrap_or(0);
        Self { lines, columns }
    }

    fn len(&self) -> usize {
        self.lines.len()
    }
}

/// Galleys of the rows shown last frame (kept in egui temp memory per
/// document): rows still visible are reused, rows scrolled away are dropped.
#[derive(Clone, Default)]
struct LineGalleys {
    pixels_per_point: u32,
    first: usize,
    galleys: Vec<Arc<Galley>>,
}

impl LineGalleys {
    /// Galleys of `rows`, laying out only rows not shown last frame.
    fn update(
        &mut self,
        ui: &egui::Ui,
        doc: &Document,
        rows: Range<usize>,
        font: &FontId,
    ) -> &[Arc<Galley>] {
        let ppp = ui.ctx().pixels_per_point().to_bits();
        if self.pixels_per_point != ppp {
            self.pixels_per_point = ppp;
            self.galleys.clear();
        }
        let cached = self.first..self.first + self.galleys.len();
        if cached != rows {
            let old = std::mem::take(&mut self.galleys);
            self.galleys = rows
                .clone()
                .filter_map(|row| match old.get(row.wrapping_sub(cached.start)) {
                    Some(galley) if cached.contains(&row) => Some(Arc::clone(galley)),
                    _ => doc.lines.get(row).map(|line| {
                        ui.painter().layout_no_wrap(
                            (*line).to_owned(),
                            font.clone(),
                            Color32::PLACEHOLDER,
                        )
                    }),
                })
                .collect();
            self.first = rows.start;
        }
        &self.galleys
    }
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
            show_document(ui, "third_party", &THIRD_PARTY_LINES, THIRD_PARTY_URL);
        });
}

/// The colormap `LICENSES.md`, shown verbatim.
pub fn show_colormap_licenses(ui: &mut egui::Ui) {
    show_document(ui, "colormaps", &COLORMAP_LINES, COLORMAP_LICENSES_URL);
}

/// Scrollable monospace view of `doc`; only the visible rows are laid out, and
/// a row's galley is reused while it stays visible.
fn show_document(ui: &mut egui::Ui, salt: &'static str, doc: &Document, url: &str) {
    let font = FontId::monospace(FONT_SIZE);
    let (row_height, glyph_width) =
        ui.fonts_mut(|f| (f.row_height(&font), f.glyph_width(&font, ' ')));
    let content = vec2(
        glyph_width * doc.columns as f32,
        row_height * doc.len() as f32,
    );
    let cache_id = egui::Id::new(("about_license_galleys", salt));
    let mut cache: LineGalleys =
        ui.data_mut(|d| std::mem::take(d.get_temp_mut_or_default(cache_id)));
    egui::ScrollArea::both()
        .id_salt(("about_license_document", salt))
        .max_height(VIEWER_HEIGHT)
        .auto_shrink([false, true])
        .show_viewport(ui, |ui, viewport| {
            let (rect, _) = ui.allocate_exact_size(content, Sense::hover());
            let rows = visible_rows(viewport, row_height, doc.len());
            for (row, galley) in rows.clone().zip(cache.update(ui, doc, rows, &font)) {
                let galley = Arc::clone(galley);
                let min = rect.min + vec2(0.0, row as f32 * row_height);
                let row_rect = Rect::from_min_size(min, vec2(content.x, row_height));
                ui.put(row_rect, Label::new(galley).halign(egui::Align::Min));
            }
        });
    ui.data_mut(|d| d.insert_temp(cache_id, cache));
    ui.hyperlink_to("View on GitHub", url);
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
        assert!(THIRD_PARTY_LINES.len() > 1000);
        assert!(COLORMAP_LINES.len() > 0);
        assert!(THIRD_PARTY_LINES.columns >= 80);
    }

    #[test]
    fn line_cache_keeps_only_the_visible_rows() {
        let doc = Document::new("a\nb\nc\nd\ne\nf");
        let font = FontId::monospace(FONT_SIZE);
        let mut cache = LineGalleys::default();
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            let first: Vec<_> = cache.update(ui, &doc, 0..3, &font).to_vec();
            assert_eq!(first.len(), 3);
            let scrolled = cache.update(ui, &doc, 2..5, &font);
            assert_eq!(scrolled.len(), 3, "rows scrolled away are dropped");
            assert!(
                Arc::ptr_eq(&scrolled[0], &first[2]),
                "visible rows are reused"
            );
            assert_eq!(scrolled[2].text(), "e");
        });
        out.textures_delta.clear();
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
