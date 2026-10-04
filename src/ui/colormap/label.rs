//! Colormap name labels (single line, elided to fit) and the hover details.

use crate::ui::hover::card::layout::line;
use crate::utils::colormap::{ColormapEntry, builtin, registry};
use egui::{Color32, FontId, Galley, Painter};
use std::sync::Arc;

/// Name font size of list rows.
pub const ROW_FONT_SIZE: f32 = 13.0;

/// Elided single-line name galley of colormap `id`, in the placeholder color so
/// painters pass the row's text color.
///
/// Laid out through egui every frame (a cache hit while unchanged), never kept
/// across frames: egui rebuilds its fonts and glyph atlas when the text options
/// change (switching between dark and light mode) or the atlas fills up, and a
/// galley from before would draw stale atlas texels as scrambled text.
pub fn name_galley(
    painter: &Painter,
    id: u32,
    font_size: f32,
    max_width: f32,
) -> Option<Arc<Galley>> {
    let font = FontId::proportional(font_size);
    registry::with_entry(id, |e| {
        line(
            painter,
            &e.name,
            font,
            Color32::PLACEHOLDER,
            max_width.max(0.0),
        )
    })
}

/// Hover details for a colormap: full name, family and kind, and license.
pub fn hover_details(ui: &mut egui::Ui, entry: &ColormapEntry) {
    ui.label(egui::RichText::new(&entry.name).strong());
    let family = entry.family.and_then(|f| builtin().families.get(f));
    ui.horizontal(|ui| {
        ui.label(family.map_or("Custom", |f| f.name.as_str()));
        ui.label(egui::RichText::new(entry.kind.label()).weak());
    });
    if let Some(family) = family {
        ui.label(egui::RichText::new(&family.license).small().weak());
    }
}
