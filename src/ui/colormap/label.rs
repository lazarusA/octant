//! Colormap name labels (single line, elided to fit) and the hover details.

use crate::ui::hover::card::layout::line;
use crate::utils::colormap::{ColormapEntry, builtin, registry};
use egui::{Color32, FontId, Galley, Painter};
use std::collections::HashMap;
use std::sync::Arc;

/// Name font size of list rows.
const ROW_FONT_SIZE: f32 = 13.0;

/// Elided name galleys of list rows, laid out once per row and reused across
/// frames until the width, the registry or the display scale changes. Galleys
/// use the placeholder color, so painters pass the row's text color.
#[derive(Default)]
pub struct NameCache {
    key: (u64, u32, u32),
    galleys: HashMap<u32, Arc<Galley>>,
}

impl NameCache {
    pub fn get(&mut self, painter: &Painter, id: u32, max_width: f32) -> Option<Arc<Galley>> {
        let key = (
            registry::generation(),
            max_width.to_bits(),
            painter.pixels_per_point().to_bits(),
        );
        if self.key != key {
            self.galleys.clear();
            self.key = key;
        }
        if let Some(galley) = self.galleys.get(&id) {
            return Some(Arc::clone(galley));
        }
        let font = FontId::proportional(ROW_FONT_SIZE);
        let galley = registry::with_entry(id, |e| {
            line(
                painter,
                &e.name,
                font,
                Color32::PLACEHOLDER,
                max_width.max(0.0),
            )
        })?;
        self.galleys.insert(id, Arc::clone(&galley));
        Some(galley)
    }
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
