//! Where the Variables, Settings and Dimensions panels sit on the canvas.
//! Docked panels line up left to right from the canvas's top-left corner in
//! that order; a panel dragged by its grip leaves the row (the others close
//! up) and keeps its own place, as a fraction of the canvas, until its grip
//! is double-clicked. Kept for the session only.

use crate::app::OctantApp;
use crate::ui::drag_grip::{self, GripAction};
use crate::ui::keyboard::regions::{self, Region};
use crate::ui::panel_header::PanelHeader;
use egui::{Id, Pos2, Rect, Response, Vec2};

/// Content width of the Dataset and Settings panels; rows stack (`flow_row`)
/// rather than widen them.
pub const PANEL_W: f32 = 280.0;

/// Gap between docked panels, and between the row and the canvas edges.
pub const GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Dataset,
    Variables,
    Settings,
    Dimensions,
}

impl Panel {
    /// The id of the panel's `egui::Area`, and so of its layer.
    pub fn area_id(self) -> Id {
        Id::unique(match self {
            Panel::Dataset => "octant_dataset_area",
            Panel::Variables => "octant_variables_area",
            Panel::Settings => "octant_settings_area",
            Panel::Dimensions => "octant_variables_panel",
        })
    }

    pub const ALL: [Panel; 4] = [
        Panel::Dataset,
        Panel::Variables,
        Panel::Settings,
        Panel::Dimensions,
    ];
}

/// Each panel's dragged top-left corner as a canvas fraction; `None` while
/// docked.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PanelPositions {
    pub dataset: Option<Pos2>,
    pub variables: Option<Pos2>,
    pub settings: Option<Pos2>,
    pub dimensions: Option<Pos2>,
}

impl PanelPositions {
    pub fn get(&self, panel: Panel) -> Option<Pos2> {
        match panel {
            Panel::Dataset => self.dataset,
            Panel::Variables => self.variables,
            Panel::Settings => self.settings,
            Panel::Dimensions => self.dimensions,
        }
    }

    fn get_mut(&mut self, panel: Panel) -> &mut Option<Pos2> {
        match panel {
            Panel::Dataset => &mut self.dataset,
            Panel::Variables => &mut self.variables,
            Panel::Settings => &mut self.settings,
            Panel::Dimensions => &mut self.dimensions,
        }
    }
}

/// The docked top-left corner after docked panels as wide as `widths`.
fn docked_origin(canvas: Rect, widths: impl Iterator<Item = f32>) -> Pos2 {
    let x: f32 = widths.map(|w| w + GAP).sum();
    canvas.left_top() + Vec2::new(GAP + x, GAP)
}

/// The width of `panel` when it is shown and docked, for the panels after it.
fn docked_width(app: &OctantApp, panel: Panel) -> Option<f32> {
    let (shown, width) = match panel {
        Panel::Dataset => (
            app.layout.show_dataset_panel,
            app.layout.dataset_overlay_width,
        ),
        Panel::Variables => (
            app.layout.show_variables_overlay,
            app.layout.variables_overlay_width,
        ),
        Panel::Settings => (
            app.layout.show_settings_panel,
            app.layout.settings_overlay_width,
        ),
        Panel::Dimensions => return None,
    };
    let docked = app.layout.panel_positions.get(panel).is_none();
    (shown && docked && width > 0.0).then_some(width)
}

/// Where `panel`'s top-left corner sits on `canvas`: its dragged place, else
/// after the docked panels before it.
pub fn origin(app: &OctantApp, panel: Panel, canvas: Rect) -> Pos2 {
    if let Some(f) = app.layout.panel_positions.get(panel) {
        return drag_grip::from_fraction(f, canvas);
    }
    let before: &[Panel] = match panel {
        Panel::Dataset => &[],
        Panel::Variables => &[Panel::Dataset],
        Panel::Settings => &[Panel::Dataset, Panel::Variables],
        Panel::Dimensions => &[Panel::Dataset, Panel::Variables, Panel::Settings],
    };
    let widths = before.iter().filter_map(|&p| docked_width(app, p));
    docked_origin(canvas, widths)
}

/// Height left for a panel's body from the cursor of `ui` down to the
/// canvas bottom, past the gap and the popup frame's bottom margin.
pub fn room_below(ui: &egui::Ui, canvas: Rect) -> f32 {
    let bottom_margin = GAP + f32::from(ui.style().spacing.menu_margin.bottom);
    canvas.bottom() - bottom_margin - ui.cursor().top()
}

/// Applies `panel`'s grip `action`, its area at `rect` on `canvas`.
fn apply_grip(app: &mut OctantApp, panel: Panel, action: GripAction, rect: Rect, canvas: Rect) {
    let pos = app.layout.panel_positions.get_mut(panel);
    *pos = drag_grip::apply(*pos, action, rect.min, canvas);
}

/// After `panel` drew as `area` on `canvas`: applies what its `header`'s
/// grip asked, registers it as a keyboard region entered at the header's
/// `entry`, and closes it when its close button was clicked.
pub fn finish(
    app: &mut OctantApp,
    panel: Panel,
    header: PanelHeader,
    area: &Response,
    canvas: Rect,
) {
    apply_grip(app, panel, header.grip, area.rect, canvas);
    if let Some(entry) = header.entry {
        regions::register(
            &area.ctx,
            Region::of(panel),
            entry,
            area.layer_id,
            Rect::EVERYTHING,
        );
    }
    if header.close {
        app.set_panel_open(&area.ctx, panel, false);
    }
}
