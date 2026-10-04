//! Kind chips and family selector for the colormap picker.

use super::search::{FamilyFilter, FilterKey};
use crate::utils::colormap::{ColormapKind, builtin};

pub fn show(ui: &mut egui::Ui, filter: &mut FilterKey) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        if ui.selectable_label(filter.kind.is_none(), "All").clicked() {
            filter.kind = None;
        }
        for kind in ColormapKind::ALL {
            let on = filter.kind == Some(kind);
            if ui.selectable_label(on, kind.label()).clicked() {
                filter.kind = if on { None } else { Some(kind) };
            }
        }
    });
    show_family_combo(ui, filter);
}

fn show_family_combo(ui: &mut egui::Ui, filter: &mut FilterKey) {
    let families = &builtin().families;
    let selected = match filter.family {
        None => "All families",
        Some(FamilyFilter::Custom) => "Custom",
        Some(FamilyFilter::Builtin(i)) => {
            families.get(i).map_or("All families", |f| f.name.as_str())
        }
    };
    egui::ComboBox::from_id_salt(("colormap_family_filter", 0))
        .selected_text(selected)
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut filter.family, None, "All families");
            for (i, family) in families.iter().enumerate() {
                ui.selectable_value(
                    &mut filter.family,
                    Some(FamilyFilter::Builtin(i)),
                    &family.name,
                )
                .on_hover_ui(|ui| {
                    ui.label(&family.attribution);
                    ui.label(egui::RichText::new(&family.license).small().weak());
                });
            }
            ui.selectable_value(&mut filter.family, Some(FamilyFilter::Custom), "Custom");
        });
}
