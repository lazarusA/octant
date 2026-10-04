//! Embedded license documents (third-party dependencies and colormap data) and a
//! virtualized viewer that lays out only the visible lines.

use crate::utils::colormap::LICENSES_TEXT;
use egui::{FontId, RichText, TextWrapMode};
use std::sync::LazyLock;

/// Notices for every crate, font and C library compiled into Octant, generated
/// by `cargo about` (see `about.toml`).
pub const THIRD_PARTY_LICENSES: &str =
    include_str!("../../../assets/licenses/THIRD_PARTY_LICENSES.md");

const THIRD_PARTY_URL: &str =
    "https://github.com/lazarusA/octant/blob/main/assets/licenses/THIRD_PARTY_LICENSES.md";
const COLORMAP_LICENSES_URL: &str =
    "https://github.com/lazarusA/octant/blob/main/assets/colormaps/LICENSES.md";

static THIRD_PARTY_LINES: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| THIRD_PARTY_LICENSES.lines().collect());
static COLORMAP_LINES: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| LICENSES_TEXT.lines().collect());

const VIEWER_HEIGHT: f32 = 260.0;
const FONT_SIZE: f32 = 10.5;

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

/// Scrollable monospace view of `lines`; only the visible rows are laid out.
fn show_document(ui: &mut egui::Ui, salt: &'static str, lines: &[&'static str], url: &str) {
    let font = FontId::monospace(FONT_SIZE);
    let row_height = ui.fonts_mut(|f| f.row_height(&font));
    egui::ScrollArea::both()
        .id_salt(("about_license_document", salt))
        .max_height(VIEWER_HEIGHT)
        .auto_shrink([false, true])
        .show_rows(ui, row_height, lines.len(), |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for line in lines.get(range).unwrap_or_default() {
                ui.add(
                    egui::Label::new(RichText::new(*line).font(font.clone()))
                        .wrap_mode(TextWrapMode::Extend),
                );
            }
        });
    ui.hyperlink_to("View on GitHub", url);
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
        assert!(!COLORMAP_LINES.is_empty());
    }
}
