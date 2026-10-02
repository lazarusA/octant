//! Size and tone reference strip for the About dialog's icon gallery.

use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};

/// Icons shown in the reference strip: one stroke-heavy, one filled, one
/// detailed, so weight differences across sizes are easy to compare.
const SAMPLE_ICONS: [Icon; 3] = [Icon::Settings, Icon::Play, Icon::Dataset];

/// Collapsible reference showing every [`IconSize`] and [`IconTone`].
pub fn show_scale_reference(ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Sizes & tones")
        .id_salt(("about_icons", "scale_reference"))
        .default_open(false)
        .show(ui, |ui| {
            egui::Grid::new(("about_icons", "size_grid"))
                .num_columns(SAMPLE_ICONS.len() + 1)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    for size in IconSize::ALL {
                        ui.label(egui::RichText::new(size_label(size)).small().monospace());
                        for icon in SAMPLE_ICONS {
                            ui.icon(icon, size);
                        }
                        ui.end_row();
                    }
                });

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for tone in IconTone::ALL {
                    ui.horizontal(|ui| {
                        ui.icon_toned(Icon::Bolt, IconSize::Sm, tone);
                        ui.label(
                            egui::RichText::new(tone.name())
                                .small()
                                .color(tone.color(ui.visuals())),
                        );
                    });
                    ui.add_space(6.0);
                }
            });
        });
}

fn size_label(size: IconSize) -> &'static str {
    match size {
        IconSize::Xs => "Xs 12",
        IconSize::Sm => "Sm 14",
        IconSize::Md => "Md 18",
        IconSize::Lg => "Lg 24",
    }
}
