use crate::app::OctantApp;
use crate::i18n::{PlotOptionsKey as K, tr};

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
    ui.horizontal(|ui| {
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

    ui.separator();
    ui.add(
        egui::Slider::new(&mut app.plot_configs.volume.quality, 0.25..=2.0)
            .text(tr(K::Quality))
            .logarithmic(true),
    )
    .on_hover_text(tr(K::QualityHover));

    if app.plot_configs.volume.algorithm == 0 || app.plot_configs.volume.algorithm >= 5 {
        ui.add(
            egui::Slider::new(&mut app.plot_configs.volume.opacity, 0.1..=10.0)
                .text(tr(K::Density)),
        );
    }

    if app.plot_configs.volume.algorithm == 1 {
        ui.add(
            egui::Slider::new(&mut app.plot_configs.volume.attenuation, 0.0..=5.0)
                .text(tr(K::Attenuation)),
        );
    }

    ui.separator();
    ui.add(
        egui::Slider::new(&mut app.plot_configs.volume.z_scale, 0.05..=10.0)
            .text(tr(K::ZScale))
            .logarithmic(true),
    );
}

pub(crate) fn show_sphere_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    let modes = [
        (0, K::ModeSmooth),
        (1, K::ModeBumpy),
        (2, K::ModeSteps),
        (3, K::ModeVoxel),
    ];

    ui.horizontal(|ui| {
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
        ui.add(
            egui::Slider::new(&mut app.plot_configs.mesh.sphere_displacement, 0.0..=5.0)
                .text(tr(K::Height)),
        );
        flip_height_checkbox(ui, &mut app.plot_configs.mesh.sphere_flip_height);
    }
}

pub(crate) fn show_surface_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    let modes = [(0, K::ModeBumpy), (1, K::ModeSteps), (2, K::ModeVoxel)];

    ui.horizontal(|ui| {
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
    ui.add(
        egui::Slider::new(&mut app.plot_configs.mesh.surface_displacement, 0.0..=5.0)
            .text(tr(K::Height)),
    );
    flip_height_checkbox(ui, &mut app.plot_configs.mesh.surface_flip_height);
}

/// Toggles heights measured down from the top value (`HeightScale::flip`).
fn flip_height_checkbox(ui: &mut egui::Ui, flip: &mut bool) {
    ui.checkbox(flip, tr(K::FlipHeights))
        .on_hover_text(tr(K::FlipHeightsHover));
}

pub(crate) fn show_point_cloud_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.add(
        egui::Slider::new(&mut app.plot_configs.point_cloud.point_size, 0.002..=0.10)
            .text(tr(K::Size)),
    );
    ui.separator();
    ui.add(
        egui::Slider::new(&mut app.plot_configs.volume.z_scale, 0.05..=10.0)
            .text(tr(K::ZScale))
            .logarithmic(true),
    );
}
