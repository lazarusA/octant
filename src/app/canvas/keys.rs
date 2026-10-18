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
use egui::{Rect, Response, Vec2};

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
    let take = |action| keymap::take_action(ctx, Action::Canvas(action), |s| gate.allows(s));
    let canvas = Canvas {
        ctx,
        rect: canvas_rect,
        plot_type,
        probe: probe::position(ctx),
    };
    let probed = canvas.probe_keys(&take, response);
    let (acted, moved) = canvas.view_keys(app, &take);
    app.nav.view_interacting |= moved;
    if probed || acted {
        ctx.request_repaint();
    }
}

/// Where the canvas keys act this frame.
struct Canvas<'a> {
    ctx: &'a egui::Context,
    rect: Rect,
    plot_type: PlotType,
    /// The probe's position, while it is on.
    probe: Option<egui::Pos2>,
}

impl Canvas<'_> {
    /// Enter starts or stops the probe; Escape stops it, else lets the
    /// canvas go. Whether either was pressed.
    fn probe_keys(
        &self,
        take: &impl Fn(CanvasAction) -> Option<Vec2>,
        response: &Response,
    ) -> bool {
        let toggle = take(CanvasAction::ToggleProbe).is_some();
        if toggle {
            let start = response.hover_pos().unwrap_or_else(|| self.rect.center());
            probe::toggle(self.ctx, start);
        }
        let leave = take(CanvasAction::LeaveProbe).is_some();
        if leave {
            match self.probe {
                Some(_) => probe::stop(self.ctx),
                None => response.surrender_focus(),
            }
        }
        toggle || leave
    }

    /// The arrows, zoom and reset keys; whether any was pressed, and
    /// whether the view moved.
    fn view_keys(
        &self,
        app: &mut OctantApp,
        take: &impl Fn(CanvasAction) -> Option<Vec2>,
    ) -> (bool, bool) {
        let (mut acted, mut moved) = (false, false);
        for (action, alt) in [(CanvasAction::Move, false), (CanvasAction::MoveAlt, true)] {
            if let Some(direction) = take(action) {
                acted = true;
                moved |= self.arrow(app, direction, alt);
            }
        }
        let zooms = [
            (CanvasAction::ZoomIn, ZOOM),
            (CanvasAction::ZoomOut, 1.0 / ZOOM),
        ];
        for (action, factor) in zooms {
            if take(action).is_some() {
                let anchor = self.probe.unwrap_or_else(|| self.rect.center());
                controller_for(self.plot_type).handle_zoom(&mut app.nav, factor, anchor, self.rect);
                (acted, moved) = (true, true);
            }
        }
        if take(CanvasAction::ResetView).is_some() {
            controller_for(self.plot_type).reset_view(&mut app.nav);
            (acted, moved) = (true, true);
        }
        (acted, moved)
    }

    /// An arrow press (`alt` with Shift) in `direction`: moves the crop box,
    /// else the probe, else the view; whether the view moved.
    fn arrow(&self, app: &mut OctantApp, direction: Vec2, alt: bool) -> bool {
        let controller = controller_for(self.plot_type);
        if app.show_crop_overlay {
            crate::ui::crop_overlay::keys::nudge(&mut app.roi_crop_box, direction, alt, self.rect);
            false
        } else if self.probe.is_some() {
            let step = probe_step(app, self.rect, self.plot_type) * if alt { FAST } else { 1.0 };
            probe::step(self.ctx, direction * step, self.rect);
            false
        } else if self.plot_type.is_3d() && !alt {
            // The view turns toward the arrow, as dragging the other way.
            controller.handle_drag(&mut app.nav, -direction * STEP);
            true
        } else {
            let far = if alt && !self.plot_type.is_3d() {
                FAST
            } else {
                1.0
            };
            controller.handle_pan(&mut app.nav, -direction * STEP * far, self.rect);
            true
        }
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
