//! The slicer's Settings rows: its mode (Off, Box, Planes, the last for
//! volumes and point clouds) on a row of its own, then, for planes, which
//! axes they cross. The cut itself is dragged on the canvas
//! (`ui::slice_gizmo`).

use crate::app::{OctantApp, SliceMode};
use crate::i18n::{self, SettingsKey as K, tr};
use crate::ui::icons::{Icon, IconSize, ToolbarButton};

/// The Slicer's label, mode select and guides eye, in a horizontal `ui`.
pub(crate) fn show_slicer(app: &mut OctantApp, ui: &mut egui::Ui) {
    let plot_type = app.effective_canvas_plot_type();
    let name = |m| match m {
        SliceMode::Off => (K::SlicerOff, K::SlicerOffHover),
        SliceMode::Box => (K::SlicerBox, K::SlicerBoxHover),
        SliceMode::Planes => (K::SlicerPlanes, K::SlicerPlanesHover),
    };
    let mut mode = app.plot_configs.slice.mode;
    ui.label(tr(K::Slicer));
    egui::ComboBox::from_id_salt("slicer_mode")
        .selected_text(tr(name(mode).0))
        .show_ui(ui, |ui| {
            for m in [SliceMode::Off, SliceMode::Box, SliceMode::Planes] {
                if m.offered_for(plot_type) {
                    let (label, hover) = name(m);
                    ui.selectable_value(&mut mode, m, tr(label))
                        .on_hover_text(tr(hover));
                }
            }
        });
    if mode != app.plot_configs.slice.mode {
        app.plot_configs.slice.set_mode(mode);
    }
    if mode != SliceMode::Off {
        guides_toggle(&mut app.plot_configs.slice.guides, ui);
    }
}

/// The mode's own row: the V, H and D planes, or whether the box cuts the
/// coastlines (`coastlines`: the plot draws them).
pub(crate) fn show_slicer_options(app: &mut OctantApp, ui: &mut egui::Ui, coastlines: bool) {
    match app.plot_configs.slice.mode {
        SliceMode::Planes => {
            ui.horizontal_wrapped(|ui| plane_toggles(app, ui));
        }
        SliceMode::Box if coastlines => {
            let cut = &mut app.plot_configs.slice.cut_coastlines;
            ui.selectable_label(*cut, tr(K::SlicerCutCoastlines))
                .on_hover_text(tr(K::SlicerCutCoastlinesHover))
                .clicked()
                .then(|| *cut = !*cut);
        }
        _ => {}
    }
}

/// The eye showing or hiding the gizmo's guides; the cut stays.
fn guides_toggle(guides: &mut bool, ui: &mut egui::Ui) {
    let (icon, hover) = if *guides {
        (Icon::Eye, K::SlicerHideGuidesHover)
    } else {
        (Icon::EyeOff, K::SlicerShowGuidesHover)
    };
    let eye = ToolbarButton::new(icon, tr(hover))
        .compact(true)
        .icon_size(IconSize::Sm);
    if ui.add(eye).clicked() {
        *guides = !*guides;
    }
}

/// One toggle per axis's plane, named by how it stands (vertical across
/// x, horizontal across y, depth across z) whatever the dimensions are;
/// the hover names the dimension.
fn plane_toggles(app: &mut OctantApp, ui: &mut egui::Ui) {
    let planes = [
        (K::SlicerPlaneV, K::SlicerPlaneVHover),
        (K::SlicerPlaneH, K::SlicerPlaneHHover),
        (K::SlicerPlaneD, K::SlicerPlaneDHover),
    ];
    for (k, (label, hover)) in planes.into_iter().enumerate() {
        let on = app.plot_configs.slice.planes[k].is_some();
        let dim = app.get_spatial_dim_index(k);
        let name = app
            .effective_variable_info()
            .and_then(|v| v.dimension_names.get(dim))
            .map_or("", String::as_str);
        let clicked = ui
            .selectable_label(on, tr(label))
            .on_hover_ui(|ui| {
                ui.label(i18n::filled(tr(hover), &[&name]));
            })
            .clicked();
        if clicked {
            app.plot_configs.slice.planes[k] = if on { None } else { Some(0.5) };
        }
    }
}
