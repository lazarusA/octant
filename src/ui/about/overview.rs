//! About tab content for the About Octant dialog.

use super::octant;
use super::types::AboutTab;
use crate::i18n::{AboutKey, tr};
use crate::ui::icons::{Icon, IconSize, UiIconExt};
use crate::utils::stack_str;

pub fn show_overview_tab(ui: &mut egui::Ui, active_tab: &mut AboutTab) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.add_space(2.0);

            // Centered header: the octant cube, its name, the title and subtitle
            ui.vertical_centered(|ui| {
                octant::show_octant_cube(ui);
                ui.add_space(4.0);
                let mut version = [0; 32];
                let title = stack_str(
                    &mut version,
                    format_args!("Octant v{}", env!("CARGO_PKG_VERSION")),
                );
                ui.heading(egui::RichText::new(title).strong());
                ui.label(
                    egui::RichText::new(tr(AboutKey::Subtitle))
                        .small()
                        .italics()
                        .color(ui.visuals().weak_text_color()),
                );
            });

            // Why the name: right under it
            ui.add_space(8.0);
            octant::show_why_octant(ui);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Core Platform Description
            ui.label(tr(AboutKey::Description));

            ui.add_space(10.0);

            show_key_specs(ui);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            show_links_and_credits(ui);

            ui.add_space(8.0);
            let mut label = [0; 64];
            let label = stack_str(
                &mut label,
                format_args!("{} ({})", tr(AboutKey::BrowseIcons), Icon::ALL.len()),
            );
            if ui
                .icon_button(Icon::Colormap, label)
                .on_hover_text(tr(AboutKey::BrowseIconsHover))
                .clicked()
            {
                *active_tab = AboutTab::Icons;
            }
        });
}

/// The framed list of headline features, each a titled row over an indented line.
fn show_key_specs(ui: &mut egui::Ui) {
    let specs = [
        (
            Icon::Bolt,
            AboutKey::HyperslabTitle,
            AboutKey::HyperslabDesc,
        ),
        (
            Icon::Icechunk,
            AboutKey::StorageTitle,
            AboutKey::StorageDesc,
        ),
        (
            Icon::Colormap,
            AboutKey::ShadersTitle,
            AboutKey::ShadersDesc,
        ),
    ];
    egui::Frame::default()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for (icon, title, desc) in specs {
                ui.horizontal(|ui| {
                    ui.icon(icon, IconSize::Sm);
                    ui.label(egui::RichText::new(tr(title)).strong());
                });
                ui.indent(("about_spec", title), |ui| ui.label(tr(desc)));
            }
        });
}

/// Project links, the good-faith notice, colormap credits and third-party licenses.
fn show_links_and_credits(ui: &mut egui::Ui) {
    // Links & Resources
    ui.heading(tr(AboutKey::LinksHeading));
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
        ui.hyperlink_to(tr(AboutKey::DocsLink), "https://docs.rs/octant");
    });
    ui.horizontal_wrapped(|ui| {
        ui.icon(Icon::Globe, IconSize::Sm);
        ui.hyperlink_to("@lazarusA", "https://github.com/lazarusA");
    });

    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(tr(AboutKey::GoodFaithNotice))
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
