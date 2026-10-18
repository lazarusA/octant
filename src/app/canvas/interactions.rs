//! Mouse, trackpad and touch interaction handling for the main plotting canvas.

use crate::app::OctantApp;
use crate::app::controllers::{PlotController, controller_for};
use crate::app::state::NavigationState;
use crate::plots::PlotType;
use egui::PointerButton;

/// How long after the first finger lands a second one still makes the touch
/// a pinch, undoing what the first finger's drag did meanwhile: fingers
/// never land at the same moment.
const PINCH_GRACE: f64 = 0.3;

/// One touch on the canvas, in egui temp memory until every finger lifts.
#[derive(Clone)]
enum TouchGesture {
    /// The view when the first finger landed.
    Started(NavigationState),
    /// Two fingers touched: nothing drags until every finger lifts.
    Pinched,
}

/// Dispatches zoom, pan, rotation, and reset interactions for 2D and 3D canvas plots.
///
/// One finger or the primary button drags (pans in 2D, orbits in 3D); two
/// fingers pinch to zoom about their center and pan as they move (a second
/// finger landing within `PINCH_GRACE` undoes the first one's drag, and
/// nothing drags until both lift); the
/// secondary or middle button, or Shift, drags to pan in 3D too; the wheel,
/// a trackpad pinch or Ctrl/Cmd+wheel zoom about the pointer.
pub fn handle_canvas_interactions(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    response: &egui::Response,
    canvas_rect: egui::Rect,
    canvas_plot_type: PlotType,
) {
    let controller = controller_for(canvas_plot_type);
    app.nav.view_interacting = false;

    if response.double_clicked() {
        controller.reset_view(&mut app.nav);
        ui.ctx().request_repaint();
    }

    if !touch_interactions(app, ui, controller, response, canvas_rect) {
        pointer_interactions(app, controller, response, canvas_rect);
    }

    if app.nav.view_interacting {
        // Smoothed scrolling can end without another frame: schedule one so the
        // settled view renders at full resolution.
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(120));
    }

    if app.nav.sphere_auto_rotate && canvas_plot_type.is_3d() {
        app.nav.sphere_rotation_y += ui.ctx().input(|i| i.stable_dt).min(0.1) * 0.15;
        ui.ctx().request_repaint();
    }
}

/// Two-finger pinches and pans; whether the touch is (or was) a pinch, so
/// one-finger drags are ignored.
fn touch_interactions(
    app: &mut OctantApp,
    ui: &egui::Ui,
    controller: &dyn PlotController,
    response: &egui::Response,
    canvas_rect: egui::Rect,
) -> bool {
    let (touching, landed, now, touch) = ui.input(|i| {
        let landed = i.pointer.press_start_time();
        (i.any_touches(), landed, i.time, i.multi_touch())
    });
    let id = response.id.with("touch_gesture");
    if !touching {
        ui.data_mut(|d| d.remove::<TouchGesture>(id));
        return false;
    }
    let gesture = ui.data(|d| d.get_temp::<TouchGesture>(id));
    // The first finger also drives the pointer: its drag is part of the
    // gesture, never an orbit or pan of its own.
    let Some(pinch) = touch.filter(|t| {
        canvas_rect.contains(t.start_pos)
            && ui.ctx().layer_id_at(t.start_pos) == Some(response.layer_id)
    }) else {
        if gesture.is_none() {
            ui.data_mut(|d| d.insert_temp(id, TouchGesture::Started(app.nav.clone())));
        }
        return matches!(gesture, Some(TouchGesture::Pinched)) || touch.is_some();
    };
    if !matches!(gesture, Some(TouchGesture::Pinched)) {
        if let Some(TouchGesture::Started(view)) = gesture
            && landed.is_some_and(|t| now - t <= PINCH_GRACE)
        {
            app.nav = view;
        }
        ui.data_mut(|d| d.insert_temp(id, TouchGesture::Pinched));
    }
    let nav = &mut app.nav;
    if pinch.zoom_delta != 1.0 {
        controller.handle_zoom(nav, pinch.zoom_delta, pinch.center_pos, canvas_rect);
    }
    if pinch.translation_delta != egui::Vec2::ZERO {
        controller.handle_pan(nav, pinch.translation_delta, canvas_rect);
    }
    nav.view_interacting = true;
    ui.ctx().request_repaint();
    true
}

/// One pointer's drags, wheel scrolling and trackpad zoom.
fn pointer_interactions(
    app: &mut OctantApp,
    controller: &dyn PlotController,
    response: &egui::Response,
    canvas_rect: egui::Rect,
) {
    let (zoom, scroll, shift) = response
        .ctx
        .input(|i| (i.zoom_delta(), i.smooth_scroll_delta.y, i.modifiers.shift));
    let nav = &mut app.nav;
    if response.dragged() {
        nav.view_interacting = true;
        let delta = response.drag_delta();
        let pans = shift
            || response.dragged_by(PointerButton::Secondary)
            || response.dragged_by(PointerButton::Middle);
        if pans {
            controller.handle_pan(nav, delta, canvas_rect);
        } else {
            controller.handle_drag(nav, delta);
        }
    }

    if response.hovered() {
        let pointer = response.hover_pos().unwrap_or_else(|| canvas_rect.center());
        if scroll != 0.0 {
            controller.handle_scroll(nav, scroll, pointer, canvas_rect);
            nav.view_interacting = true;
        }
        if zoom != 1.0 {
            controller.handle_zoom(nav, zoom, pointer, canvas_rect);
            nav.view_interacting = true;
        }
        if nav.view_interacting {
            response.ctx.request_repaint();
        }
    }
}
