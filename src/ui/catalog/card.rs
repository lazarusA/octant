use crate::app::StoreKind;
use crate::catalog::CatalogEntry;
use crate::i18n::{DatasetKey, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::utils::stack_str;

pub fn render_entry_card(
    ui: &mut egui::Ui,
    entry: &CatalogEntry,
    trimmed_url: &str,
    is_mobile: bool,
) -> bool {
    let ome = crate::catalog::MICROSCOPY_CATALOG
        .iter()
        .any(|e| e.key == entry.key);
    let (badge_icon, badge_name, badge_tone) = match entry.store_kind {
        StoreKind::RemoteZarr if ome => (Icon::Globe, "OME-Zarr", IconTone::Success),
        StoreKind::RemoteZarr => (Icon::Globe, "Zarr", IconTone::Accent),
        StoreKind::RemoteIcechunk => (Icon::Icechunk, "Icechunk", IconTone::Info),
        StoreKind::RemoteGeoTiff | StoreKind::LocalGeoTiff => {
            (Icon::Folder, "GeoTIFF/COG", IconTone::Success)
        }
        StoreKind::ProceduralVolume4D => (
            Icon::PlotVolume,
            tr(DatasetKey::BadgeVolume4D),
            IconTone::Warning,
        ),
        StoreKind::ProceduralRandom => (
            Icon::PlotPlane,
            tr(DatasetKey::BadgeMatrix2D),
            IconTone::Warning,
        ),
        _ => (Icon::Folder, tr(DatasetKey::BadgeStore), IconTone::Muted),
    };
    let mut badge_buf = [0u8; 96];
    let badge_bracket = stack_str(&mut badge_buf, format_args!("[{badge_name}]"));

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
                    ui.icon_toned(badge_icon, IconSize::Xs, badge_tone);
                    ui.label(
                        egui::RichText::new(badge_bracket)
                            .strong()
                            .small()
                            .color(badge_tone.color(ui.visuals())),
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
                    .icon_button(Icon::DropTray, tr(DatasetKey::SelectAndLoadDataset))
                    .clicked()
                {
                    clicked_load = true;
                }
            } else {
                // Desktop layout: Side-by-side header with right-aligned button
                ui.horizontal(|ui| {
                    ui.icon_toned(badge_icon, IconSize::Xs, badge_tone);
                    ui.label(
                        egui::RichText::new(badge_bracket)
                            .strong()
                            .small()
                            .color(badge_tone.color(ui.visuals())),
                    );
                    ui.label(egui::RichText::new(entry.label).strong().size(14.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .icon_button(Icon::DropTray, tr(DatasetKey::SelectAndLoad))
                            .clicked()
                        {
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
