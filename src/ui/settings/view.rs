use super::support::OptionSupport;
use crate::app::OctantApp;
use crate::ui::icons::{Icon, UiIconExt};

/// Camera controls for 3D plots and the hover card toggle, for every plot.
pub(crate) fn show_view_controls(app: &mut OctantApp, ui: &mut egui::Ui, support: &OptionSupport) {
    ui.horizontal(|ui| {
        if support.camera.is_yes() {
            ui.checkbox(&mut app.sphere_auto_rotate, "Auto Rotate");
            if ui
                .icon_button(Icon::Reset, "Reset View")
                .on_hover_text("Reset 3D camera orientation")
                .clicked()
            {
                app.sphere_rotation_x = 0.25;
                app.sphere_rotation_y = 0.0;
                app.sphere_zoom = 2.5;
            }
        }
        ui.selectable_label(app.show_hover_card, "Hover Card")
            .on_hover_text(if app.show_hover_card {
                "Hide hover card"
            } else {
                "Show hover card"
            })
            .clicked()
            .then(|| app.show_hover_card = !app.show_hover_card);
    });
}
