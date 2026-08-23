use crate::app::OctantApp;

pub fn show_about_window(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_about_window {
        return;
    }

    let mut open = app.show_about_window;

    let response = egui::Window::new("About Octant")
        .open(&mut open)
        .default_width(400.0)
        .max_width(440.0)
        .resizable(false)
        .collapsible(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            egui::Frame::default()
                .inner_margin(egui::Margin::symmetric(14, 4))
                .show(ui, |ui| {
                    ui.add_space(4.0);

                    // Centered Header with Animated Octant Cube Widget & Title
                    ui.vertical_centered(|ui| {
                        crate::ui::hero::draw_octant_widget(
                            ui,
                            42.0,
                            [-1.0, -1.0, -1.0],
                            0.0,
                            1.0,
                        );
                        ui.add_space(4.0);
                        ui.heading(
                            egui::RichText::new(format!("Octant v{}", env!("CARGO_PKG_VERSION")))
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("N-Dimensional Data Explorer")
                                .small()
                                .italics()
                                .color(ui.visuals().weak_text_color()),
                        );
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Core Platform Description
                    use egui::special_emojis::{OS_APPLE, OS_LINUX, OS_WINDOWS};
                    ui.label(format!(
                        "Octant is an interactive viewer for N-dimensional datasets with native support for local and cloud object storage, Zarr (v2/v3), and Icechunk. Built in Rust with GPU-accelerated rendering via WGPU. Octant runs natively on {}{}{}.",
                        OS_APPLE, OS_LINUX, OS_WINDOWS
                    ));

                    ui.add_space(10.0);

                    // Key Specs
                    egui::Frame::default()
                        .fill(ui.visuals().extreme_bg_color)
                        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 4.0;
                            ui.label(egui::RichText::new("⚡ Hyperslab Slicing").strong());
                            ui.label("   Async LRU chunk cache & multi-resolution pyramids.");
                            ui.label(egui::RichText::new("🧊 Zarr & Icechunk Native").strong());
                            ui.label("   Local, S3, GCS, Azure, and HTTP streaming backends.");
                            ui.label(
                                egui::RichText::new("🎨 Hardware-Accelerated WGPU Shaders").strong(),
                            );
                            ui.label(
                                "   2D Flatmaps, 3D Spheres, Elevation Surfaces, Volumes & 1D Profiles.",
                            );
                        });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Links
                    ui.heading("Links");
                    ui.add_space(4.0);

                    use egui::special_emojis::GITHUB;
                    ui.horizontal_wrapped(|ui| {
                        ui.hyperlink_to(
                            format!("{GITHUB} github.com/lazarusA/octant"),
                            "https://github.com/lazarusA/octant",
                        );
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.hyperlink_to("📓 octant documentation", "https://docs.rs/octant");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.hyperlink_to("👤 @lazarusA", "https://github.com/lazarusA");
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    ui.vertical_centered(|ui| {
                        ui.small("Licensed under MIT or Apache-2.0");
                    });
                });
        });

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        open = false;
    }

    if let Some(r) = response
        && let Some(pos) = ctx.input(|i| i.pointer.interact_pos())
        && ctx.input(|i| i.pointer.primary_pressed())
        && !r.response.rect.contains(pos)
    {
        open = false;
    }

    app.show_about_window = open;
}
