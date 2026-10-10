use super::support::OptionSupport;
use crate::app::OctantApp;
use crate::ui::icons::{Icon, UiIconExt};

/// Camera controls for 3D plots and the hover card toggle, for every plot.
pub(crate) fn show_view_controls(app: &mut OctantApp, ui: &mut egui::Ui, support: &OptionSupport) {
    ui.horizontal(|ui| {
        if support.camera.is_yes() {
            ui.checkbox(&mut app.nav.sphere_auto_rotate, "Auto Rotate");
            if ui
                .icon_button(Icon::Reset, "Reset View")
                .on_hover_text("Reset 3D camera orientation")
                .clicked()
            {
                app.nav.reset_3d_camera();
            }
        }
        ui.selectable_label(app.layout.show_hover_card, "Hover Card")
            .on_hover_text(if app.layout.show_hover_card {
                "Hide hover card"
            } else {
                "Show hover card"
            })
            .clicked()
            .then(|| app.layout.show_hover_card = !app.layout.show_hover_card);
    });
}
