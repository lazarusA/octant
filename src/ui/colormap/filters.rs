//! Kind chips and family chips for the colormap picker. Both are inline
//! selectable labels: a dropdown would open its own popup layer, and a click
//! there counts as "outside" the colormap popup and closes it.

use super::search::{FamilyFilter, FilterKey};
use crate::i18n::{ColormapKey as K, tr};
use crate::utils::colormap::{ColormapKind, builtin};

pub fn show(ui: &mut egui::Ui, filter: &mut FilterKey) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        if ui
            .selectable_label(filter.kind.is_none(), tr(K::FilterAll))
            .clicked()
        {
            filter.kind = None;
        }
        for kind in ColormapKind::ALL {
            let on = filter.kind == Some(kind);
            if ui.selectable_label(on, kind.label()).clicked() {
                filter.kind = if on { None } else { Some(kind) };
            }
        }
    });
    show_family_chips(ui, filter);
}

/// Collapsible "Family: …" section with one chip per family.
fn show_family_chips(ui: &mut egui::Ui, filter: &mut FilterKey) {
    let families = &builtin().families;
    let selected = match filter.family {
        None => tr(K::FilterAll),
        Some(FamilyFilter::Custom) => tr(K::FamilyCustom),
        Some(FamilyFilter::Builtin(i)) => families
            .get(i)
            .map_or(tr(K::FilterAll), |f| f.name.as_str()),
    };
    let mut buf = [0u8; 160];
    let title = crate::i18n::fill(&mut buf, tr(K::FamilyTitle), &[&selected]);
    egui::CollapsingHeader::new(title)
        .id_salt(("colormap_family_filter", 0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                ui.selectable_value(&mut filter.family, None, tr(K::FilterAll));
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
                ui.selectable_value(
                    &mut filter.family,
                    Some(FamilyFilter::Custom),
                    tr(K::FamilyCustom),
                );
            });
        });
}
