//! The plate browser's acquisition select and the picked well's fields.

use super::PlateAction;
use crate::app::{PlateState, WellFields};
use crate::i18n::{DatasetKey, tr};
use crate::utils::stack_str;

/// The name of acquisition `id` of `layout` (all of them for `None`), its
/// number when it has no name.
fn acquisition_name<'a>(
    layout: &'a crate::data::ome::OmePlate,
    id: Option<u64>,
    buf: &'a mut [u8; 24],
) -> &'a str {
    match id.and_then(|id| layout.acquisitions.iter().find(|a| a.id == id)) {
        Some(a) => match a.name.as_deref() {
            Some(name) => name,
            None => stack_str(buf, format_args!("{}", a.id)),
        },
        None => tr(DatasetKey::PlateAllAcquisitions),
    }
}

/// The acquisition whose fields are listed, when the plate has several.
pub(super) fn acquisition_select(ui: &mut egui::Ui, plate: &mut PlateState) {
    let layout = std::sync::Arc::clone(&plate.plate);
    if layout.acquisitions.len() < 2 {
        return;
    }
    let mut buf = [0u8; 24];
    let selected = acquisition_name(&layout, plate.acquisition, &mut buf);
    ui.horizontal(|ui| {
        ui.label(tr(DatasetKey::PlateAcquisition));
        egui::ComboBox::from_id_salt("plate_acquisition")
            .selected_text(selected)
            .truncate()
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                let all = tr(DatasetKey::PlateAllAcquisitions);
                ui.selectable_value(&mut plate.acquisition, None, all);
                for a in &layout.acquisitions {
                    let mut buf = [0u8; 24];
                    let label = acquisition_name(&layout, Some(a.id), &mut buf);
                    ui.selectable_value(&mut plate.acquisition, Some(a.id), label);
                }
            });
    });
}

/// Whether `opened` is field `field` of well `well` of the plate at `url`.
pub(super) fn is_field(opened: Option<&str>, url: &str, well: &str, field: &str) -> bool {
    opened
        .and_then(|o| o.strip_prefix(url))
        .and_then(|o| o.strip_prefix('/'))
        .and_then(|o| o.strip_prefix(well))
        .and_then(|o| o.strip_prefix('/'))
        == Some(field)
}

/// The picked well's fields; a click opens one.
pub(super) fn field_list(ui: &mut egui::Ui, plate: &mut PlateState) -> Option<PlateAction> {
    let layout = std::sync::Arc::clone(&plate.plate);
    let Some(well) = plate.well.and_then(|i| layout.wells.get(i)) else {
        ui.label(
            egui::RichText::new(tr(DatasetKey::PlatePickWell))
                .small()
                .weak(),
        );
        return None;
    };
    let fields = match plate.well_fields(&well.path) {
        WellFields::Loading => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new(tr(DatasetKey::PlateLoadingFields))
                        .small()
                        .weak(),
                );
            });
            return None;
        }
        WellFields::Failed(error) => {
            ui.label(egui::RichText::new(error.as_str()).small().weak());
            return None;
        }
        WellFields::Ready(fields) => std::sync::Arc::clone(fields),
    };
    let opened = plate.opened.as_deref();
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        ui.label(well.path.as_str());
        ui.label(
            egui::RichText::new(tr(DatasetKey::PlateFields))
                .small()
                .weak(),
        );
        let mut any = false;
        for (n, field) in fields.fields_of(plate.acquisition).enumerate() {
            any = true;
            let is_open = is_field(opened, &plate.url, &well.path, &field.path);
            let mut buf = [0u8; 16];
            let label = stack_str(&mut buf, format_args!("{}", n + 1));
            if ui
                .selectable_label(is_open, label)
                .on_hover_text(field.path.as_str())
                .clicked()
            {
                action = Some(PlateAction::OpenField(
                    well.path.clone(),
                    field.path.clone(),
                ));
            }
        }
        if !any {
            ui.label(
                egui::RichText::new(tr(DatasetKey::PlateNoFields))
                    .small()
                    .weak(),
            );
        }
    });
    action
}
