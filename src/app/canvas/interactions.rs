//! Mouse and gesture interaction handling for the main plotting canvas.

use crate::app::OctantApp;
use crate::app::controllers::controller_for;
use crate::plots::PlotType;

/// Dispatches zoom, pan, rotation, and reset interactions for 2D and 3D canvas plots.
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

    if response.dragged() {
        app.nav.view_interacting = true;
        controller.handle_drag(&mut app.nav, response.drag_delta());
    }

    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            let mouse_pos = response.hover_pos().unwrap_or_else(|| canvas_rect.center());
            controller.handle_scroll(&mut app.nav, scroll, mouse_pos, canvas_rect.center());
            app.nav.view_interacting = true;
            ui.ctx().request_repaint();
        }
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
