//! About tab content for the About Octant dialog.

use super::types::AboutTab;
use crate::app::OctantApp;
use crate::ui::icons::{Icon, IconSize, UiIconExt};

pub fn show_overview_tab(_app: &mut OctantApp, ui: &mut egui::Ui, active_tab: &mut AboutTab) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.add_space(2.0);

            // Centered Header with Animated Octant Cube Widget & Title
            ui.vertical_centered(|ui| {
                crate::ui::hero::draw_octant_widget(ui, 42.0, [-1.0, -1.0, -1.0], 0.0, 1.0);
                ui.add_space(4.0);
                ui.heading(
                    egui::RichText::new(format!("Octant v{}", env!("CARGO_PKG_VERSION"))).strong(),
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
            ui.label(
                "Octant is an interactive viewer for N-dimensional datasets with native support for local and cloud object storage, Zarr (v2/v3), and Icechunk. Built in Rust with GPU-accelerated rendering via WGPU. Octant runs natively on macOS, Linux, and Windows.",
            );

            ui.add_space(10.0);

            // Key Specs
            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.horizontal(|ui| {
                        ui.icon(Icon::Bolt, IconSize::Sm);
                        ui.label(egui::RichText::new("Hyperslab Slicing").strong());
                    });
                    ui.label("   Async LRU chunk cache & multi-resolution pyramids.");
                    ui.horizontal(|ui| {
                        ui.icon(Icon::Icechunk, IconSize::Sm);
                        ui.label(egui::RichText::new("Zarr & Icechunk Native").strong());
                    });
                    ui.label("   Local, S3, GCS, Azure, and HTTP streaming backends.");
                    ui.horizontal(|ui| {
                        ui.icon(Icon::Colormap, IconSize::Sm);
                        ui.label(
                            egui::RichText::new("Hardware-Accelerated WGPU Shaders").strong(),
                        );
                    });
                    ui.label(
                        "   2D Flatmaps, 3D Spheres, Elevation Surfaces, Volumes & 1D Profiles.",
                    );
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            show_links_and_credits(ui);

            ui.add_space(8.0);
            if ui
                .icon_button(Icon::Colormap, crate::utils::stack_str(
                    &mut [0; 48],
                    format_args!("Browse Native Vector Icons ({})", Icon::ALL.len()),
                ))
                .on_hover_text("Explore procedural vector icons in popover")
                .clicked()
            {
                *active_tab = AboutTab::Icons;
            }
        });
}

/// Project links, the good-faith notice, colormap credits and third-party licenses.
fn show_links_and_credits(ui: &mut egui::Ui) {
    // Links & Resources
    ui.heading("Links & Resources");
    ui.add_space(4.0);

    ui.horizontal_wrapped(|ui| {
        ui.icon(Icon::Catalog, IconSize::Sm);
        ui.hyperlink_to(
            "github.com/lazarusA/octant",
            "https://github.com/lazarusA/octant",
        );
    });
    ui.horizontal_wrapped(|ui| {
        ui.icon(Icon::VariableDoc, IconSize::Sm);
        ui.hyperlink_to("octant documentation", "https://docs.rs/octant");
    });
    ui.horizontal_wrapped(|ui| {
        ui.icon(Icon::Globe, IconSize::Sm);
        ui.hyperlink_to("@lazarusA", "https://github.com/lazarusA");
    });

    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(
            "Octant credits third-party work in good faith. If you believe something \
             is missing or incorrectly attributed, please let us know and we will \
             correct or remove it.",
        )
        .small()
        .weak(),
    );
    ui.hyperlink_to(
        egui::RichText::new("github.com/lazarusA/octant/issues").small(),
        "https://github.com/lazarusA/octant/issues",
    );
    super::credits::show_colormap_credits(ui);
    super::licenses::show_third_party_licenses(ui);
}
