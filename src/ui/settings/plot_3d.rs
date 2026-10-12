use crate::app::OctantApp;
use crate::i18n::{PlotOptionsKey as K, tr};
use crate::ui::flow_row::{SliderLabel, labeled_slider};
use crate::ui::number_input::LocalizedInput;

/// Each volume algorithm's id and name.
const ALGORITHMS: [(u32, K); 8] = [
    (0, K::AlgoDvr),
    (1, K::AlgoMip),
    (2, K::AlgoMinIp),
    (3, K::AlgoXray),
    (4, K::AlgoLabelSurface),
    (5, K::AlgoAbsorption),
    (6, K::AlgoAdditive),
    (7, K::AlgoIndexed),
];

pub(crate) fn show_volume_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    algorithm_row(app, ui);

    ui.separator();
    labeled_slider(
        ui,
        "quality",
        SliderLabel::After(tr(K::Quality)),
        egui::Slider::new(&mut app.plot_configs.volume.quality, 0.25..=2.0)
            .localized()
            .logarithmic(true),
    )
    .on_hover_text(tr(K::QualityHover));

    if app.plot_configs.volume.algorithm == 0 || app.plot_configs.volume.algorithm >= 5 {
        labeled_slider(
            ui,
            "density",
            SliderLabel::After(tr(K::Density)),
            egui::Slider::new(&mut app.plot_configs.volume.opacity, 0.1..=10.0).localized(),
        );
    }

    if app.plot_configs.volume.algorithm == 1 {
        labeled_slider(
            ui,
            "attenuation",
            SliderLabel::After(tr(K::Attenuation)),
            egui::Slider::new(&mut app.plot_configs.volume.attenuation, 0.0..=5.0).localized(),
        );
    }

    ui.separator();
    labeled_slider(
        ui,
        "volume_z_scale",
        SliderLabel::After(tr(K::ZScale)),
        egui::Slider::new(&mut app.plot_configs.volume.z_scale, 0.05..=10.0)
            .localized()
            .logarithmic(true),
    );
}

/// The volume algorithm's name, opening the menu of algorithms.
fn algorithm_row(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(tr(K::Algorithm))
                .small()
                .color(ui.visuals().weak_text_color()),
        );
        let algo_label = match app.plot_configs.volume.algorithm {
            id @ 0..=6 => ALGORITHMS[id as usize].1,
            _ => K::AlgoIndexed,
        };
        ui.menu_button(egui::RichText::new(tr(algo_label)).small(), |ui| {
            for (id, label) in ALGORITHMS {
                if ui
                    .selectable_label(app.plot_configs.volume.algorithm == id, tr(label))
                    .clicked()
                {
                    app.plot_configs.volume.algorithm = id;
                    ui.close();
                }
            }
        });
    });
}

pub(crate) fn show_sphere_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    let modes = [
        (0, K::ModeSmooth),
        (1, K::ModeBumpy),
        (2, K::ModeSteps),
        (3, K::ModeVoxel),
    ];

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (id, label) in modes {
            if ui
                .selectable_label(
                    app.plot_configs.mesh.sphere_mode == id,
                    egui::RichText::new(tr(label)),
                )
                .clicked()
            {
                app.plot_configs.mesh.sphere_mode = id;
            }
        }
    });

    if app.plot_configs.mesh.sphere_mode > 0 {
        ui.separator();
        labeled_slider(
            ui,
            "sphere_height",
            SliderLabel::After(tr(K::Height)),
            egui::Slider::new(&mut app.plot_configs.mesh.sphere_displacement, 0.0..=5.0)
                .localized(),
        );
        flip_height_checkbox(ui, &mut app.plot_configs.mesh.sphere_flip_height);
    }
}

pub(crate) fn show_surface_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    let modes = [(0, K::ModeBumpy), (1, K::ModeSteps), (2, K::ModeVoxel)];

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (id, label) in modes {
            if ui
                .selectable_label(
                    app.plot_configs.mesh.surface_mode == id,
                    egui::RichText::new(tr(label)),
                )
                .clicked()
            {
                app.plot_configs.mesh.surface_mode = id;
            }
        }
    });

    ui.separator();
    labeled_slider(
        ui,
        "surface_height",
        SliderLabel::After(tr(K::Height)),
        egui::Slider::new(&mut app.plot_configs.mesh.surface_displacement, 0.0..=5.0).localized(),
    );
    flip_height_checkbox(ui, &mut app.plot_configs.mesh.surface_flip_height);
}

/// Toggles heights measured down from the top value (`HeightScale::flip`).
fn flip_height_checkbox(ui: &mut egui::Ui, flip: &mut bool) {
    ui.checkbox(flip, tr(K::FlipHeights))
        .on_hover_text(tr(K::FlipHeightsHover));
}

pub(crate) fn show_point_cloud_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    labeled_slider(
        ui,
        "point_size",
        SliderLabel::After(tr(K::Size)),
        egui::Slider::new(&mut app.plot_configs.point_cloud.point_size, 0.002..=0.10).localized(),
    );
    ui.separator();
    labeled_slider(
        ui,
        "point_cloud_z_scale",
        SliderLabel::After(tr(K::ZScale)),
        egui::Slider::new(&mut app.plot_configs.volume.z_scale, 0.05..=10.0)
            .localized()
            .logarithmic(true),
    );
}
