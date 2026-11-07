use super::GROUP_GAP;
use super::support::OptionSupport;
use crate::app::OctantApp;
use crate::i18n::{ColormapKey, Key, PlotOptionsKey, SettingsKey as K, tr};
use crate::ui::axes::{AxisStyle, MinorPlacement};
use crate::ui::flow_row::{SliderLabel, labeled_slider};
use crate::ui::icons::{Icon, UiIconExt};
use crate::ui::number_input::LocalizedInput;

/// One row of toggles (the heatmap's aspect ratio, the 3D camera, hover
/// card and scale bar), the slicer's mode row and its options' row, the axes
/// and the colorbars' box transparency while one shows, each group set apart.
pub(crate) fn show_view_controls(app: &mut OctantApp, ui: &mut egui::Ui, support: &OptionSupport) {
    toggles_row(app, ui, support);
    if support.slice.is_yes() {
        ui.add_space(GROUP_GAP);
        ui.horizontal_wrapped(|ui| super::slicer::show_slicer(app, ui));
        super::slicer::show_slicer_options(app, ui, support.coastlines.is_yes());
    }
    if support.axes.is_yes() {
        ui.add_space(GROUP_GAP);
        axis_controls(&mut app.layout.axis_style, ui);
    }
    let colorbars = app
        .layers
        .iter()
        .any(|l| l.colorbar.shown && crate::ui::colorbar::can_show(app, l.id()));
    if colorbars {
        ui.add_space(GROUP_GAP);
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

/// The aspect ratio, 3D camera, hover card and scale bar, as
/// far as the plot uses them, wrapping within the panel.
fn toggles_row(app: &mut OctantApp, ui: &mut egui::Ui, support: &OptionSupport) {
    ui.horizontal_wrapped(|ui| {
        if support.aspect_ratio.is_yes() {
            toggle(
                ui,
                &mut app.layout.enforce_data_aspect_ratio,
                PlotOptionsKey::AspectRatio,
                PlotOptionsKey::AspectRatioHover,
            );
        }
        if support.camera.is_yes() {
            let rotate = &mut app.nav.sphere_auto_rotate;
            ui.selectable_label(*rotate, tr(K::AutoRotate))
                .clicked()
                .then(|| *rotate = !*rotate);
            if ui
                .icon_button(Icon::Reset, tr(K::ResetView))
                .on_hover_text(tr(K::ResetViewHover))
                .clicked()
            {
                app.nav.reset_3d_camera();
            }
        }
        let hover_card = app.layout.show_hover_card;
        toggle(
            ui,
            &mut app.layout.show_hover_card,
            K::HoverCard,
            if hover_card {
                K::HideHoverCardHover
            } else {
                K::ShowHoverCardHover
            },
        );
        if support.scale_bar.is_yes() {
            let shown = app.layout.show_scale_bar;
            toggle(
                ui,
                &mut app.layout.show_scale_bar,
                K::ScaleBar,
                if shown {
                    K::HideScaleBarHover
                } else {
                    K::ShowScaleBarHover
                },
            );
        }
    });
}

/// The axes' major and minor rows: ticks and grid lines each on or off, and
/// on the minor row where minors go.
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
        if style.minors().is_some() {
            minor_placement(style, ui);
        }
    });
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
fn toggle(ui: &mut egui::Ui, on: &mut bool, label: impl Into<Key>, hover: impl Into<Key>) {
    if ui
        .selectable_label(*on, tr(label))
        .on_hover_text(tr(hover))
        .clicked()
    {
        *on = !*on;
    }
}
