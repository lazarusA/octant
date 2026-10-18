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
    canvas_rect: Rect,
    plot_type: PlotType,
) {
    let ctx = ui.ctx();
    if ctx.input(|i| i.pointer.delta() != Vec2::ZERO) {
        probe::stop(ctx);
    }
    let gate = Gate::new(app, ctx, true);
    if !gate.allows(Scope::Canvas) {
        return;
    }
    if response.has_focus() {
        focus::claim_arrows(ctx, response.id);
    }
    let acted = std::cell::Cell::new(false);
    let take = |action| {
        let taken = keymap::take_action(ctx, Action::Canvas(action), |s| gate.allows(s));
        acted.set(acted.get() || taken.is_some());
        taken
    };
    let controller = controller_for(plot_type);
    let probe_at = probe::position(ctx);
    let mut moved = false;

    if take(CanvasAction::ToggleProbe).is_some() {
        let start = response.hover_pos().unwrap_or_else(|| canvas_rect.center());
        probe::toggle(ctx, start);
    }
    if take(CanvasAction::LeaveProbe).is_some() {
        match probe_at {
            Some(_) => probe::stop(ctx),
            None => response.surrender_focus(),
        }
    }
    for (action, alt) in [(CanvasAction::Move, false), (CanvasAction::MoveAlt, true)] {
        let Some(direction) = take(action) else {
            continue;
        };
        if app.show_crop_overlay {
            crate::ui::crop_overlay::keys::nudge(
                &mut app.roi_crop_box,
                direction,
                alt,
                canvas_rect,
            );
        } else if probe_at.is_some() {
            let step = probe_step(app, canvas_rect, plot_type) * if alt { FAST } else { 1.0 };
            probe::step(ctx, direction * step, canvas_rect);
        } else if plot_type.is_3d() && !alt {
            // The view turns toward the arrow, as dragging the other way.
            controller.handle_drag(&mut app.nav, -direction * STEP);
            moved = true;
        } else {
            let far = if alt && !plot_type.is_3d() { FAST } else { 1.0 };
            controller.handle_pan(&mut app.nav, -direction * STEP * far, canvas_rect);
            moved = true;
        }
    }

    let anchor = probe_at.unwrap_or_else(|| canvas_rect.center());
    for (action, factor) in [
        (CanvasAction::ZoomIn, ZOOM),
        (CanvasAction::ZoomOut, 1.0 / ZOOM),
    ] {
        if take(action).is_some() {
            controller.handle_zoom(&mut app.nav, factor, anchor, canvas_rect);
            moved = true;
        }
    }
    if take(CanvasAction::ResetView).is_some() {
        controller.reset_view(&mut app.nav);
        moved = true;
    }
    if moved {
        app.nav.view_interacting = true;
    }
    if acted.get() {
        ctx.request_repaint();
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
