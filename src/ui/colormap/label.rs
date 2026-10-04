//! Single-line colormap names, elided with `…` to fit, and the hover card text.

use crate::utils::colormap::{ColormapEntry, builtin};
use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Color32, FontId, Galley};
use std::sync::Arc;

/// Lays out `text` on one line, cut with `…` when wider than `max_width`.
pub fn elided(
    ui: &egui::Ui,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(text.to_owned(), TextFormat::simple(font, color));
    job.wrap = TextWrapping::truncate_at_width(max_width.max(0.0));
    ui.fonts_mut(|f| f.layout_job(job))
}

/// Hover details for a colormap: full name, family, kind and license.
pub fn hover_details(ui: &mut egui::Ui, entry: &ColormapEntry) {
    ui.label(egui::RichText::new(&entry.name).strong());
    let family = entry.family.and_then(|f| builtin().families.get(f));
    let family_name = family.map_or("Custom", |f| f.name.as_str());
    ui.label(format!("{family_name} · {}", entry.kind.label()));
    if let Some(family) = family {
        ui.label(egui::RichText::new(&family.license).small().weak());
    }
}
