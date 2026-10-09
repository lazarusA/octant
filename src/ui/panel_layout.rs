//! Where the Variables, Settings and Dimensions panels sit on the canvas.
//! Docked panels line up left to right from the canvas's top-left corner in
//! that order; a panel dragged by its grip leaves the row (the others close
//! up) and keeps its own place, as a fraction of the canvas, until its grip
//! is double-clicked. Kept for the session only.

use crate::app::OctantApp;
use crate::ui::drag_grip::{self, GripAction};
use egui::{Pos2, Rect, Vec2};

/// Gap between docked panels, and between the row and the canvas edges.
pub const GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Variables,
    Settings,
    Dimensions,
}

/// Each panel's dragged top-left corner as a canvas fraction; `None` while
/// docked.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PanelPositions {
    pub variables: Option<Pos2>,
    pub settings: Option<Pos2>,
    pub dimensions: Option<Pos2>,
}

impl PanelPositions {
    pub fn get(&self, panel: Panel) -> Option<Pos2> {
        match panel {
            Panel::Variables => self.variables,
            Panel::Settings => self.settings,
            Panel::Dimensions => self.dimensions,
        }
    }

    fn get_mut(&mut self, panel: Panel) -> &mut Option<Pos2> {
        match panel {
            Panel::Variables => &mut self.variables,
            Panel::Settings => &mut self.settings,
            Panel::Dimensions => &mut self.dimensions,
        }
    }
}

/// The docked top-left corner after the docked panels `widths` before it.
pub fn docked_origin(canvas: Rect, widths: &[f32]) -> Pos2 {
    let x: f32 = widths.iter().map(|w| w + GAP).sum();
    canvas.left_top() + Vec2::new(GAP + x, GAP)
}

/// The width of `panel` when it is shown and docked, for the panels after it.
fn docked_width(app: &OctantApp, panel: Panel) -> Option<f32> {
    let (shown, width) = match panel {
        Panel::Variables => (app.show_variables_overlay, app.variables_overlay_width),
        Panel::Settings => (app.show_settings_panel, app.settings_overlay_width),
        Panel::Dimensions => return None,
    };
    let docked = app.panel_positions.get(panel).is_none();
    (shown && docked && width > 0.0).then_some(width)
}

/// Where `panel`'s top-left corner sits on `canvas`: its dragged place, else
/// after the docked panels before it.
pub fn origin(app: &OctantApp, panel: Panel, canvas: Rect) -> Pos2 {
    if let Some(f) = app.panel_positions.get(panel) {
        return drag_grip::from_fraction(f, canvas);
    }
    let before: &[Panel] = match panel {
        Panel::Variables => &[],
        Panel::Settings => &[Panel::Variables],
        Panel::Dimensions => &[Panel::Variables, Panel::Settings],
    };
    let mut widths = [0.0; 2];
    let mut n = 0;
    for &p in before {
        if let Some(w) = docked_width(app, p) {
            widths[n] = w;
            n += 1;
        }
    }
    docked_origin(canvas, &widths[..n])
}

/// Applies `panel`'s grip `action`, its area at `rect` on `canvas`.
pub fn apply_grip(app: &mut OctantApp, panel: Panel, action: GripAction, rect: Rect, canvas: Rect) {
    let pos = app.panel_positions.get_mut(panel);
    *pos = drag_grip::apply(*pos, action, rect.min, canvas);
}
