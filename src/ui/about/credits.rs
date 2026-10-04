//! Colormap credits: every bundled family with its license and source.

use crate::utils::colormap::builtin;

pub fn show_colormap_credits(ui: &mut egui::Ui) {
    let families = &builtin().families;
    egui::CollapsingHeader::new("Colormap credits")
        .id_salt(("about_colormap_credits", 0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Bundled colormaps keep the licenses of their authors.")
                    .small()
                    .weak(),
            );
            for family in families {
                ui.add_space(3.0);
                ui.horizontal_wrapped(|ui| {
                    ui.hyperlink_to(&family.name, &family.source);
                    ui.label(egui::RichText::new(&family.license).small().strong());
                });
                ui.label(egui::RichText::new(&family.attribution).small().weak());
            }
            ui.add_space(4.0);
            show_license_texts(ui);
        });
}

/// The bundled `LICENSES.md`, embedded in the binary and shown verbatim.
fn show_license_texts(ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("License texts")
        .id_salt(("about_colormap_license_texts", 0))
        .show(ui, super::licenses::show_colormap_licenses);
}
