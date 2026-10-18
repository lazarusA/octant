//! The plate browser in the Dataset panel: the plate's wells on a rows x
//! columns grid, its acquisitions, and the picked well's fields; picking a
//! field opens it as the active dataset while the plate stays browsable.

mod fields;
#[cfg(test)]
mod tests;

use fields::{acquisition_select, field_list};

use crate::app::OctantApp;
use crate::app::PlateState;
use crate::ui::keyboard::focus;

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
    /// Field path of the well at an index.
    OpenField(usize, String),
}

/// The plate browser, while a plate is open.
pub fn show_plate_browser(app: &mut OctantApp, ui: &mut egui::Ui) {
    let Some(plate) = &mut app.plate else {
        return;
    };
    if let Some(name) = plate.plate.name.as_deref() {
        ui.label(egui::RichText::new(name).strong());
    }
    let picked = well_grid(ui, plate);
    acquisition_select(ui, plate);
    match field_list(ui, plate).or(picked) {
        Some(PlateAction::PickWell(i)) => {
            if let Some(plate) = &mut app.plate {
                plate.well = Some(i);
            }
        }
        Some(PlateAction::OpenField(well, field)) => app.open_plate_field(well, &field),
        None => {}
    }
}

/// The side of a well cell for `columns` columns in `width`.
pub(crate) fn cell_side(width: f32, columns: usize) -> f32 {
    ((width - ROW_NAME_W) / columns.max(1) as f32).clamp(CELL_MIN, CELL_MAX)
}

/// Where a plate's wells are drawn: the grid's rect and its cell side.
struct Grid {
    rect: egui::Rect,
    side: f32,
}

impl Grid {
    /// The cell of row `r`, column `c` (row 0 holds the column names).
    fn cell(&self, r: usize, c: usize) -> egui::Rect {
        let offset = egui::vec2(
            ROW_NAME_W + self.side * c as f32,
            self.side * (r + 1) as f32,
        );
        egui::Rect::from_min_size(self.rect.min + offset, egui::Vec2::splat(self.side))
    }
}

/// The wells on their grid; a click (or Enter) picks one, and the arrow
/// keys move keyboard focus to the next well along a row or column.
fn well_grid(ui: &mut egui::Ui, plate: &PlateState) -> Option<PlateAction> {
    let layout = &plate.plate;
    let (rows, columns) = (layout.rows.len(), layout.columns.len());
    if rows == 0 || columns == 0 {
        return None;
    }
    let side = cell_side(ui.available_width(), columns);
    let size = egui::vec2(ROW_NAME_W + side * columns as f32, side * (rows + 1) as f32);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let grid = Grid { rect, side };
    grid_names(ui, &grid, layout);
    let painter = ui.painter_at(rect);
    let well_id = |i: usize| ui.scope_id().with(("plate_well", i));
    let mut action = None;
    let mut focused = None;
    for (i, well) in layout.wells.iter().enumerate() {
        if well.row >= rows || well.column >= columns {
            continue;
        }
        let cell = grid.cell(well.row, well.column).shrink(1.0);
        let response = ui.interact(cell, well_id(i), egui::Sense::click());
        paint_well(ui, &painter, plate, i, &response);
        if response.has_focus() {
            focused = Some(i);
            focus::claim_arrows(ui.ctx(), response.id);
        }
        if response.clicked() {
            action = Some(PlateAction::PickWell(i));
        }
        response.on_hover_text(well.path.as_str());
    }
    let step = focus::pressed_arrow(ui.ctx());
    if let Some(next) = focused
        .zip(step)
        .and_then(|(at, d)| neighbor(layout, at, d))
    {
        focus::request(ui.ctx(), well_id(next));
    }
    action
}

/// Well `i`'s cell: picked, hovered or plain fill, the opened well's
/// outline and the keyboard focus ring.
fn paint_well(
    ui: &egui::Ui,
    painter: &egui::Painter,
    plate: &PlateState,
    i: usize,
    response: &egui::Response,
) {
    let visuals = ui.visuals();
    let fill = if plate.well == Some(i) {
        visuals.selection.bg_fill
    } else if response.hovered() {
        visuals.widgets.hovered.bg_fill
    } else {
        visuals.widgets.inactive.bg_fill
    };
    let cell = response.rect;
    painter.rect_filled(cell, 2.0, fill);
    if plate.opened.as_ref().is_some_and(|(well, _)| *well == i) {
        painter.rect_stroke(
            cell,
            2.0,
            visuals.selection.stroke,
            egui::StrokeKind::Inside,
        );
    }
    focus::paint_ring(ui, response, 2.0);
}

/// The well next to well `at` in `direction` along its row or column: the
/// nearest one there, as plates may leave wells out.
fn neighbor(
    layout: &crate::data::ome::OmePlate,
    at: usize,
    direction: egui::Vec2,
) -> Option<usize> {
    let from = layout.wells.get(at)?;
    let along = |row: usize, column: usize| {
        let (dr, dc) = (
            row as f32 - from.row as f32,
            column as f32 - from.column as f32,
        );
        let ahead = if direction.x != 0.0 {
            dr == 0.0 && dc * direction.x > 0.0
        } else {
            dc == 0.0 && dr * direction.y > 0.0
        };
        ahead.then_some((dr.abs() + dc.abs()) as usize)
    };
    let wells = layout.wells.iter().enumerate();
    wells
        .filter_map(|(i, w)| along(w.row, w.column).map(|d| (d, i)))
        .min()
        .map(|(_, i)| i)
}

/// The column names above the grid and the row names left of it, every few
/// only when cells are too small to name each.
fn grid_names(ui: &egui::Ui, grid: &Grid, layout: &crate::data::ome::OmePlate) {
    let side = grid.side;
    let every = if side >= NAMED_CELL {
        1
    } else {
        (NAMED_CELL / side).ceil() as usize
    };
    let painter = ui.painter_at(grid.rect);
    let font = egui::FontId::proportional((side * 0.6).clamp(7.0, 11.0));
    let muted = ui.visuals().weak_text_color();
    let name = |at: egui::Pos2, text: &str| {
        painter.text(at, egui::Align2::CENTER_CENTER, text, font.clone(), muted);
    };
    for (c, column) in layout.columns.iter().enumerate().step_by(every) {
        name(grid.cell(0, c).center() - egui::vec2(0.0, side), column);
    }
    for (r, row) in layout.rows.iter().enumerate().step_by(every) {
        let y = grid.cell(r, 0).center().y;
        name(egui::pos2(grid.rect.min.x + ROW_NAME_W * 0.5, y), row);
    }
}
