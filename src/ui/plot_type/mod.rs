//! Picking how the variable is drawn: the top bar's plot type menu, and the
//! centered picker the `P` shortcut opens (`picker`). Both list the plot
//! types `options` allows, the others with the reason they cannot be picked.

pub mod options;
pub mod picker;
#[cfg(test)]
mod sheet;
#[cfg(test)]
mod tests;

use crate::app::OctantApp;
use crate::i18n::{PlotKey, fill, tr};
use crate::plots::PlotType;
use crate::ui::icons::{Icon, IconSize, IconTone, ToolbarButton, UiIconExt};
use crate::ui::keyboard::keymap::Action;
use options::{PlotOption, blocked_text, plot_options};

/// Map a PlotType to its corresponding vector Icon.
#[inline]
pub fn plot_type_icon(plot_type: PlotType) -> Icon {
    match plot_type {
        PlotType::Heatmap => Icon::PlotPlane,
        PlotType::Line => Icon::PlotLine,
        PlotType::Sphere => Icon::PlotGlobe,
        PlotType::Surface => Icon::PlotSurface,
        PlotType::Volume => Icon::PlotVolume,
        PlotType::PointCloud => Icon::PlotPointCloud,
    }
}

/// The plot type button and its popup; the button's response.
pub fn show_plot_type_menu(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    compact: bool,
) -> egui::Response {
    let current_icon = plot_type_icon(app.selected.plot_type);
    let current_label = app.selected.plot_type.display_name();
    let button_response = ui.add(
        ToolbarButton::new(current_icon, current_label)
            .compact(compact)
            .owns_popup()
            .shortcut(Action::PlotTypePicker),
    );
    egui::Popup::from_toggle_button_response(&button_response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| menu_contents(app, ui));
    button_response
}

/// One row per plot type: pickable ones select it, the others say why not.
fn menu_contents(app: &mut OctantApp, ui: &mut egui::Ui) {
    let options = plot_options(app);
    ui.set_min_width(220.0);
    ui.label(
        egui::RichText::new(tr(PlotKey::SelectProjection))
            .small()
            .weak(),
    );
    ui.separator();
    for PlotOption { plot_type, blocked } in options {
        let icon = plot_type_icon(plot_type);
        let label = plot_type.display_name();
        let Some(blocked) = blocked else {
            let selected = app.selected.plot_type == plot_type;
            let clicked = ui
                .horizontal(|ui| {
                    ui.icon(icon, IconSize::Sm);
                    ui.selectable_label(selected, label).clicked()
                })
                .inner;
            if clicked {
                app.switch_plot_type(plot_type);
            }
            continue;
        };
        ui.horizontal(|ui| {
            ui.icon_toned(icon, IconSize::Sm, IconTone::Muted);
            let mut reason_buf = [0; 256];
            let reason = blocked_text(&mut reason_buf, blocked);
            let mut buf = [0; 384];
            let text = fill(&mut buf, tr(PlotKey::DisabledItem), &[&label, &reason]);
            ui.add_enabled(false, egui::Label::new(text));
        });
    }
}
