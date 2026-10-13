//! The plate browser in the Dataset panel: the plate's wells on a rows x
//! columns grid, its acquisitions, and the picked well's fields; picking a
//! field opens it as the active dataset while the plate stays browsable.

mod fields;
#[cfg(test)]
mod tests;

#[cfg(test)]
use fields::is_field;
use fields::{acquisition_select, field_list};

use crate::app::OctantApp;
use crate::app::PlateState;

/// Width of the row names column.
const ROW_NAME_W: f32 = 18.0;
/// Bounds of a well cell's side.
const CELL_MIN: f32 = 5.0;
const CELL_MAX: f32 = 22.0;
/// Below this side, column names show every few columns only.
const NAMED_CELL: f32 = 14.0;

/// What the browser asked for this frame, applied after it is drawn.
pub(super) enum PlateAction {
    PickWell(usize),
    OpenField(String, String),
}

/// The plate browser, while a plate is open.
pub fn show_plate_browser(app: &mut OctantApp, ui: &mut egui::Ui) {
    let Some(plate) = &mut app.plate else {
        return;
    };
    let mut action = None;
    if let Some(name) = plate.plate.name.as_deref() {
        ui.label(egui::RichText::new(name).strong());
    }
    if let Some(a) = well_grid(ui, plate) {
        action = Some(a);
    }
    acquisition_select(ui, plate);
    if let Some(a) = field_list(ui, plate) {
        action = Some(a);
    }
    match action {
        Some(PlateAction::PickWell(i)) => {
            if let Some(plate) = &mut app.plate {
                plate.well = Some(i);
            }
        }
        Some(PlateAction::OpenField(well, field)) => app.open_plate_field(&well, &field),
        None => {}
    }
}

/// The side of a well cell for `columns` columns in `width`.
pub(crate) fn cell_side(width: f32, columns: usize) -> f32 {
    ((width - ROW_NAME_W) / columns.max(1) as f32).clamp(CELL_MIN, CELL_MAX)
}

/// The wells on their grid; a click picks one.
fn well_grid(ui: &mut egui::Ui, plate: &PlateState) -> Option<PlateAction> {
    let layout = &plate.plate;
    let (rows, columns) = (layout.rows.len(), layout.columns.len());
    if rows == 0 || columns == 0 {
        return None;
    }
    let side = cell_side(ui.available_width(), columns);
    let label_every = if side >= NAMED_CELL {
        1
    } else {
        (NAMED_CELL / side).ceil() as usize
    };
    let size = egui::vec2(ROW_NAME_W + side * columns as f32, side * (rows + 1) as f32);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    let font = egui::FontId::proportional((side * 0.6).clamp(7.0, 11.0));
    let muted = visuals.weak_text_color();
    let cell = |r: usize, c: usize| {
        egui::Rect::from_min_size(
            rect.min + egui::vec2(ROW_NAME_W + side * c as f32, side * (r + 1) as f32),
            egui::vec2(side, side),
        )
    };
    for (c, column) in layout.columns.iter().enumerate().step_by(label_every) {
        let at = cell(0, c).center() - egui::vec2(0.0, side);
        painter.text(
            at,
            egui::Align2::CENTER_CENTER,
            column.as_str(),
            font.clone(),
            muted,
        );
    }
    for (r, row) in layout.rows.iter().enumerate() {
        let at = egui::pos2(rect.min.x + ROW_NAME_W * 0.5, cell(r, 0).center().y);
        painter.text(
            at,
            egui::Align2::CENTER_CENTER,
            row.as_str(),
            font.clone(),
            muted,
        );
    }
    let opened_well = plate.opened.as_deref();
    let mut action = None;
    for (i, well) in layout.wells.iter().enumerate() {
        if well.row >= rows || well.column >= columns {
            continue;
        }
        let r = cell(well.row, well.column).shrink(1.0);
        let id = ui.id().with(("plate_well", i));
        let response = ui.interact(r, id, egui::Sense::click());
        let picked = plate.well == Some(i);
        let fill = if picked {
            visuals.selection.bg_fill
        } else if response.hovered() {
            visuals.widgets.hovered.bg_fill
        } else {
            visuals.widgets.inactive.bg_fill
        };
        painter.rect_filled(r, 2.0, fill);
        let opened = opened_well.is_some_and(|url| {
            url.strip_prefix(plate.url.as_str())
                .and_then(|rest| rest.strip_prefix('/'))
                .and_then(|rest| rest.strip_prefix(well.path.as_str()))
                .is_some_and(|rest| rest.starts_with('/'))
        });
        if opened {
            painter.rect_stroke(r, 2.0, visuals.selection.stroke, egui::StrokeKind::Inside);
        }
        if response.clicked() {
            action = Some(PlateAction::PickWell(i));
        }
        response.on_hover_text(well.path.as_str());
    }
    action
}
