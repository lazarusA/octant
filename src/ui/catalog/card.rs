use crate::app::StoreKind;
use crate::catalog::CatalogEntry;
use crate::ui::icons::{Icon, UiIconExt};

pub fn render_entry_card(
    ui: &mut egui::Ui,
    entry: &CatalogEntry,
    trimmed_url: &str,
    is_mobile: bool,
) -> bool {
    let (badge_icon, badge_bracket, badge_color) = match entry.store_kind {
        StoreKind::RemoteZarr => (Icon::Globe, "[Zarr]", ui.visuals().selection.bg_fill),
        StoreKind::RemoteIcechunk => (
            Icon::Icechunk,
            "[Icechunk]",
            ui.visuals().widgets.active.bg_fill,
        ),
        StoreKind::RemoteGeoTiff | StoreKind::LocalGeoTiff => (
            Icon::Folder,
            "[GeoTIFF/COG]",
            ui.visuals().widgets.active.bg_fill,
        ),
        StoreKind::ProceduralVolume4D => (
            Icon::PlotVolume,
            "[4D Volume]",
            ui.visuals().widgets.hovered.bg_fill,
        ),
        StoreKind::ProceduralRandom => (
            Icon::PlotPlane,
            "[2D Matrix]",
            ui.visuals().widgets.hovered.bg_fill,
        ),
        _ => (
            Icon::Folder,
            "[Store]",
            ui.visuals().widgets.noninteractive.fg_stroke.color,
        ),
    };

    let mut clicked_load = false;

    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            if is_mobile {
                // Mobile layout: Stacked
                ui.horizontal(|ui| {
                    ui.icon_colored(badge_icon, 13.0, badge_color);
                    ui.label(
                        egui::RichText::new(badge_bracket)
                            .strong()
                            .small()
                            .color(badge_color),
                    );
                    ui.label(egui::RichText::new(entry.label).strong().size(13.5));
                });

                if !entry.subtitle.is_empty() {
                    ui.label(egui::RichText::new(entry.subtitle).small().italics());
                }

                ui.add_space(2.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(trimmed_url)
                            .small()
                            .monospace()
                            .color(ui.visuals().hyperlink_color),
                    )
                    .wrap_mode(egui::TextWrapMode::Wrap),
                );

                ui.add_space(4.0);
                if ui
                    .icon_button(Icon::DropTray, "Select & Load Dataset")
                    .clicked()
                {
                    clicked_load = true;
                }
            } else {
                // Desktop layout: Side-by-side header with right-aligned button
                ui.horizontal(|ui| {
                    ui.icon_colored(badge_icon, 13.0, badge_color);
                    ui.label(
                        egui::RichText::new(badge_bracket)
                            .strong()
                            .small()
                            .color(badge_color),
                    );
                    ui.label(egui::RichText::new(entry.label).strong().size(14.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.icon_button(Icon::DropTray, "Select & Load").clicked() {
                            clicked_load = true;
                        }
                    });
                });

                if !entry.subtitle.is_empty() {
                    ui.label(egui::RichText::new(entry.subtitle).small().italics());
                }

                ui.add_space(2.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(trimmed_url)
                            .small()
                            .monospace()
                            .color(ui.visuals().hyperlink_color),
                    )
                    .wrap_mode(egui::TextWrapMode::Wrap),
                );
            }
        });

    clicked_load
}
