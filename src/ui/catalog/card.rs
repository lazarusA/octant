use crate::app::StoreKind;
use crate::catalog::CatalogEntry;
use crate::i18n::{DatasetKey, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::utils::stack_str;

/// The card of `entry`; its load button's response.
pub fn render_entry_card(
    ui: &mut egui::Ui,
    entry: &CatalogEntry,
    trimmed_url: &str,
    is_mobile: bool,
) -> egui::Response {
    let (badge_icon, badge_name, badge_tone) = badge(entry);
    let mut badge_buf = [0u8; 96];
    let badge = stack_str(&mut badge_buf, format_args!("[{badge_name}]"));
    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Mobile stacks the load button under the card; desktop puts it
            // right of the title.
            let beside_title = ui.horizontal(|ui| {
                ui.icon_toned(badge_icon, IconSize::Xs, badge_tone);
                let color = badge_tone.color(ui.visuals());
                ui.label(egui::RichText::new(badge).strong().small().color(color));
                if cfg!(target_arch = "wasm32") && entry.desktop_only() {
                    desktop_only_chip(ui);
                }
                let size = if is_mobile { 13.5 } else { 14.0 };
                ui.label(egui::RichText::new(entry.label).strong().size(size));
                let right = egui::Layout::right_to_left(egui::Align::Center);
                (!is_mobile).then(|| {
                    ui.with_layout(right, |ui| {
                        load_button(ui, entry, DatasetKey::SelectAndLoad)
                    })
                    .inner
                })
            });
            if !entry.subtitle.is_empty() {
                ui.label(egui::RichText::new(entry.subtitle).small().italics());
            }
            ui.add_space(2.0);
            let url = egui::RichText::new(trimmed_url)
                .small()
                .monospace()
                .color(ui.visuals().hyperlink_color);
            ui.add(egui::Label::new(url).wrap_mode(egui::TextWrapMode::Wrap));
            beside_title.inner.unwrap_or_else(|| {
                ui.add_space(4.0);
                load_button(ui, entry, DatasetKey::SelectAndLoadDataset)
            })
        })
        .inner
}

/// The card's load button, labeled `key`; in the browser, a desktop-only
/// entry's button says why it cannot load.
fn load_button(ui: &mut egui::Ui, entry: &CatalogEntry, key: DatasetKey) -> egui::Response {
    let button = ui.icon_button(Icon::DropTray, tr(key));
    if cfg!(target_arch = "wasm32") && entry.desktop_only() {
        button.on_hover_text(tr(DatasetKey::DesktopOnlyHint))
    } else {
        button
    }
}

/// The browser's "[Desktop only]" chip of entries whose servers it cannot read.
fn desktop_only_chip(ui: &mut egui::Ui) {
    let tone = IconTone::Warning;
    ui.icon_toned(Icon::Lock, IconSize::Xs, tone);
    let mut buf = [0u8; 96];
    let text = stack_str(
        &mut buf,
        format_args!("[{}]", tr(DatasetKey::BadgeDesktopOnly)),
    );
    let color = tone.color(ui.visuals());
    ui.label(egui::RichText::new(text).strong().small().color(color))
        .on_hover_text(tr(DatasetKey::DesktopOnlyHint));
}

/// The entry's badge: icon, format name and tone.
fn badge(entry: &CatalogEntry) -> (Icon, &'static str, IconTone) {
    match entry.store_kind {
        StoreKind::RemoteZarr if entry.is_ome() => (Icon::Globe, "OME-Zarr", IconTone::Success),
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
    }
}
