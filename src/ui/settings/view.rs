use super::support::OptionSupport;
use crate::app::OctantApp;
use crate::i18n::{ColormapKey, SettingsKey as K, tr};
use crate::ui::axes::{AxisStyle, MinorPlacement};
use crate::ui::flow_row::{SliderLabel, labeled_slider};
use crate::ui::icons::{Icon, UiIconExt};
use crate::ui::number_input::LocalizedInput;

/// Camera controls for 3D plots and the hover card and scale bar toggles,
/// for every plot, then the colorbars' box transparency while one shows.
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
    if support.axes.is_yes() {
        axis_controls(&mut app.layout.axis_style, ui);
    }
    let colorbars = app
        .layers
        .iter()
        .any(|l| l.colorbar.shown && crate::ui::colorbar::can_show(app, l.id()));
    if colorbars {
        labeled_slider(
            ui,
            "colorbar_transparency",
            SliderLabel::After(tr(ColormapKey::BoxTransparency)),
            egui::Slider::new(&mut app.layout.colorbar_transparency, 0.0..=1.0)
                .localized()
                .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
        );
    }
}

/// The axes' major and minor rows: ticks and grid lines each on or off, and
/// where minors go.
fn axis_controls(style: &mut AxisStyle, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.label(tr(K::AxisMajor));
        toggle(ui, &mut style.major_ticks, K::AxisTicks, K::AxisTicksHover);
        toggle(ui, &mut style.major_grid, K::AxisGrid, K::AxisGridHover);
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(tr(K::AxisMinor));
        toggle(ui, &mut style.minor_ticks, K::AxisTicks, K::AxisTicksHover);
        toggle(ui, &mut style.minor_grid, K::AxisGrid, K::AxisGridHover);
    });
    if style.minors().is_some() {
        minor_placement(style, ui);
    }
}

/// Where minor ticks and grid lines go: cell centers or round values.
fn minor_placement(style: &mut AxisStyle, ui: &mut egui::Ui) {
    let placement = |p| match p {
        MinorPlacement::Cells => (K::MinorCells, K::MinorCellsHover),
        MinorPlacement::Round => (K::MinorRound, K::MinorRoundHover),
    };
    egui::ComboBox::from_id_salt("minor_placement")
        .selected_text(tr(placement(style.minor_placement).0))
        .show_ui(ui, |ui| {
            for p in [MinorPlacement::Cells, MinorPlacement::Round] {
                let (label, hover) = placement(p);
                ui.selectable_value(&mut style.minor_placement, p, tr(label))
                    .on_hover_text(tr(hover));
            }
        });
}

/// A selectable label flipping `on`.
fn toggle(ui: &mut egui::Ui, on: &mut bool, label: K, hover: K) {
    if ui
        .selectable_label(*on, tr(label))
        .on_hover_text(tr(hover))
        .clicked()
    {
        *on = !*on;
    }
}
