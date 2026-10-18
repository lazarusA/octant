//! Keys on the plot, while nothing or the canvas holds focus: the arrows pan
//! a 2D plot or orbit a 3D one (Shift pans further, or pans in 3D), `+` and
//! `-` zoom, `0` resets the view. Enter starts the keyboard probe, whose
//! cursor the arrows then move cell by cell; with the crop box shown they
//! move it, and Shift+arrows resize it. Every move goes through the plot
//! type's controller, as a drag, pan or zoom by the pointer would.

use crate::app::OctantApp;
use crate::app::controllers::controller_for;
use crate::plots::PlotType;
use crate::ui::hover::probe;
use crate::ui::keyboard::dispatch::Gate;
use crate::ui::keyboard::focus;
use crate::ui::keyboard::keymap::{self, Action, CanvasAction, Scope};
use egui::{Rect, Vec2};

/// Points one arrow press pans a plot or orbits a 3D one.
const STEP: f32 = 40.0;
/// How much further Shift moves a 2D plot or the probe.
const FAST: f32 = 4.0;
/// Zoom factor of one `+` press.
const ZOOM: f32 = 1.25;
/// Points one arrow press moves the probe over a 3D plot.
const PROBE_STEP_3D: f32 = 8.0;

/// Runs this frame's canvas keys; call after the pointer's interactions.
pub fn handle_canvas_keys(
    app: &mut OctantApp,
    ui: &egui::Ui,
    response: &egui::Response,
    rect: Rect,
    plot_type: PlotType,
) {
    let ctx = ui.ctx();
    // The probe drives the hover card: gone with it, or once the mouse moves.
    if !app.layout.show_hover_card || ctx.input(|i| i.pointer.delta() != Vec2::ZERO) {
        probe::stop(ctx);
    }
    let gate = Gate::new(app, ctx, true);
    if !gate.allows(Scope::Canvas) {
        return;
    }
    if response.has_focus() {
        focus::claim_arrows(ctx, response.id);
    }
    if !keymap::any_pressed(ctx) {
        return;
    }
    let take = |action| keymap::take_action(ctx, Action::Canvas(action), |s| gate.allows(s));
    let probe_at = probe::position(ctx);
    if take(CanvasAction::ToggleProbe).is_some() && app.layout.show_hover_card {
        probe::toggle(ctx, response.hover_pos().unwrap_or_else(|| rect.center()));
    }
    if take(CanvasAction::LeaveProbe).is_some() {
        match probe_at {
            Some(_) => probe::stop(ctx),
            None => response.surrender_focus(),
        }
    }
    let mut moved = false;
    for (action, alt) in [(CanvasAction::Move, false), (CanvasAction::MoveAlt, true)] {
        if let Some(direction) = take(action) {
            let probing = probe_at.is_some();
            moved |= arrow(app, ctx, direction, alt, rect, plot_type, probing);
        }
    }
    let controller = controller_for(plot_type);
    let zooms = [
        (CanvasAction::ZoomIn, ZOOM),
        (CanvasAction::ZoomOut, 1.0 / ZOOM),
    ];
    for (action, factor) in zooms {
        if take(action).is_some() {
            let anchor = probe_at.unwrap_or_else(|| rect.center());
            controller.handle_zoom(&mut app.nav, factor, anchor, rect);
            moved = true;
        }
    }
    if take(CanvasAction::ResetView).is_some() {
        controller.reset_view(&mut app.nav);
        moved = true;
    }
    app.nav.view_interacting |= moved;
    ctx.request_repaint();
}

/// An arrow press (`alt` with Shift) in `direction` on the canvas at `rect`
/// showing `plot_type` (`probing` while the probe is on): moves the crop
/// box, else the probe, else the view; whether the view moved.
fn arrow(
    app: &mut OctantApp,
    ctx: &egui::Context,
    direction: Vec2,
    alt: bool,
    rect: Rect,
    plot_type: PlotType,
    probing: bool,
) -> bool {
    let controller = controller_for(plot_type);
    if app.show_crop_overlay {
        crate::ui::crop_overlay::keys::nudge(&mut app.roi_crop_box, direction, alt, rect);
        false
    } else if probing {
        let step = probe_step(app, rect, plot_type) * if alt { FAST } else { 1.0 };
        probe::step(ctx, direction * step, rect);
        false
    } else if plot_type.is_3d() && !alt {
        // The view turns toward the arrow, as dragging the other way.
        controller.handle_drag(&mut app.nav, -direction * STEP);
        true
    } else {
        let far = if alt && !plot_type.is_3d() { FAST } else { 1.0 };
        controller.handle_pan(&mut app.nav, -direction * STEP * far, rect);
        true
    }
}

/// One probe step: a data cell of a 2D plot as drawn, else a few points.
fn probe_step(app: &OctantApp, canvas_rect: Rect, plot_type: PlotType) -> Vec2 {
    let matrix = app.layers.base.data.matrix.as_ref();
    match matrix.filter(|_| !plot_type.is_3d()) {
        Some(m) if m.width > 0 && m.height > 0 => {
            let plot = super::compute_viewport_uniforms(app, canvas_rect, plot_type).plot_rect;
            let cells = Vec2::new(m.width as f32, m.height as f32);
            (plot.size() / cells).max(Vec2::splat(1.0))
        }
        _ => Vec2::splat(PROBE_STEP_3D),
    }
}
