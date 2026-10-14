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

/// The picked well's fields; a click opens one.
pub(super) fn field_list(ui: &mut egui::Ui, plate: &mut PlateState) -> Option<PlateAction> {
    let layout = std::sync::Arc::clone(&plate.plate);
    let Some((index, well)) = plate.well.and_then(|i| Some((i, layout.wells.get(i)?))) else {
        ui.label(muted(DatasetKey::PlatePickWell));
        return None;
    };
    let WellFields::Ready(fields) = plate.well_fields(&well.path) else {
        well_status(ui, plate, &well.path);
        return None;
    };
    let fields = std::sync::Arc::clone(fields);
    let opened = match &plate.opened {
        Some((w, field)) if *w == index => Some(field.as_str()),
        _ => None,
    };
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        ui.label(well.path.as_str());
        ui.label(muted(DatasetKey::PlateFields));
        let mut any = false;
        for (n, field) in fields.fields_of(plate.acquisition) {
            any = true;
            let is_open = opened == Some(field.path.as_str());
            let mut buf = [0u8; 16];
            let label = stack_str(&mut buf, format_args!("{}", n + 1));
            if ui
                .selectable_label(is_open, label)
                .on_hover_text(field.path.as_str())
                .clicked()
            {
                action = Some(PlateAction::OpenField(index, field.path.clone()));
            }
        }
        if !any {
            ui.label(muted(DatasetKey::PlateNoFields));
        }
    });
    action
}

/// A note in small, weak text.
fn muted(key: DatasetKey) -> egui::RichText {
    egui::RichText::new(tr(key)).small().weak()
}

/// The well at `path` while its fields load, or a link retrying it when
/// they could not be read.
fn well_status(ui: &mut egui::Ui, plate: &mut PlateState, path: &str) {
    if matches!(plate.well_fields(path), WellFields::Failed) {
        let retry = egui::RichText::new(tr(DatasetKey::PlateWellUnreadable)).small();
        if ui.link(retry).clicked() {
            plate.retry_well(path);
        }
    } else {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(muted(DatasetKey::PlateLoadingFields));
        });
    }
}
