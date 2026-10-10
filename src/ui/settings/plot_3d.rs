use crate::app::OctantApp;

pub(crate) fn show_volume_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Algorithm")
                .small()
                .color(ui.visuals().weak_text_color()),
        );
        let algo_label = match app.plot_configs.volume.algorithm {
            0 => "Volume Raymarching (DVR)",
            1 => "Maximum Intensity (MIP)",
            2 => "Minimum Intensity (MinIP)",
            3 => "Average Projection (X-ray)",
            4 => "Categorical Label Surface",
            5 => "Absorption RGBA",
            6 => "Additive RGBA",
            _ => "Indexed RGBA",
        };
        ui.menu_button(egui::RichText::new(algo_label).small(), |ui| {
            let algos = [
                (0, "Volume Raymarching (DVR)"),
                (1, "Maximum Intensity (MIP)"),
                (2, "Minimum Intensity (MinIP)"),
                (3, "Average Projection (X-ray)"),
                (4, "Categorical Label Surface"),
                (5, "Absorption RGBA"),
                (6, "Additive RGBA"),
                (7, "Indexed RGBA"),
            ];
            for (id, label) in algos {
                if ui
                    .selectable_label(app.plot_configs.volume.algorithm == id, label)
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
            .text("Quality")
            .logarithmic(true),
    )
    .on_hover_text("Samples per voxel along each ray");

    if app.plot_configs.volume.algorithm == 0 || app.plot_configs.volume.algorithm >= 5 {
        ui.add(egui::Slider::new(&mut app.plot_configs.volume.opacity, 0.1..=10.0).text("Density"));
    }

    if app.plot_configs.volume.algorithm == 1 {
        ui.add(
            egui::Slider::new(&mut app.plot_configs.volume.attenuation, 0.0..=5.0)
                .text("Attenuation"),
        );
    }

    ui.separator();
    ui.add(
        egui::Slider::new(&mut app.plot_configs.volume.z_scale, 0.05..=10.0)
            .text("Z-Scale")
            .logarithmic(true),
    );
}

pub(crate) fn show_sphere_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    let modes: [(u32, &str); 4] = [(0, "Smooth"), (1, "Bumpy"), (2, "Steps"), (3, "Voxel")];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (id, label) in modes {
            if ui
                .selectable_label(
                    app.plot_configs.mesh.sphere_mode == id,
                    egui::RichText::new(label),
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
                .text("Height"),
        );
    }
}

pub(crate) fn show_surface_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    let modes: [(u32, &str); 3] = [(0, "Bumpy"), (1, "Steps"), (2, "Voxel")];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (id, label) in modes {
            if ui
                .selectable_label(
                    app.plot_configs.mesh.surface_mode == id,
                    egui::RichText::new(label),
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
            .text("Height"),
    );
}

pub(crate) fn show_point_cloud_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.add(
        egui::Slider::new(&mut app.plot_configs.point_cloud.point_size, 0.002..=0.10).text("Size"),
    );
    ui.separator();
    ui.add(
        egui::Slider::new(&mut app.plot_configs.volume.z_scale, 0.05..=10.0)
            .text("Z-Scale")
            .logarithmic(true),
    );
}
