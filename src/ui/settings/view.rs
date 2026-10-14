use super::support::OptionSupport;
use crate::app::OctantApp;
use crate::i18n::{SettingsKey as K, tr};
use crate::ui::icons::{Icon, UiIconExt};

/// Camera controls for 3D plots and the hover card toggle, for every plot.
pub(crate) fn show_view_controls(app: &mut OctantApp, ui: &mut egui::Ui, support: &OptionSupport) {
    ui.horizontal_wrapped(|ui| {
        if support.camera.is_yes() {
            ui.checkbox(&mut app.nav.sphere_auto_rotate, tr(K::AutoRotate));
            if ui
                .icon_button(Icon::Reset, tr(K::ResetView))
                .on_hover_text(tr(K::ResetViewHover))
                .clicked()
            {
                app.nav.reset_3d_camera();
            }
        }
        ui.selectable_label(app.layout.show_hover_card, tr(K::HoverCard))
            .on_hover_text(tr(if app.layout.show_hover_card {
                K::HideHoverCardHover
            } else {
                K::ShowHoverCardHover
            }))
            .clicked()
            .then(|| app.layout.show_hover_card = !app.layout.show_hover_card);
        if support.scale_bar.is_yes() {
            ui.selectable_label(app.layout.show_scale_bar, tr(K::ScaleBar))
                .on_hover_text(tr(if app.layout.show_scale_bar {
                    K::HideScaleBarHover
                } else {
                    K::ShowScaleBarHover
                }))
                .clicked()
                .then(|| app.layout.show_scale_bar = !app.layout.show_scale_bar);
        }
    });
}
