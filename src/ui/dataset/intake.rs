use crate::app::OctantApp;
use crate::app::StoreKind;
use crate::ui::icons::{Icon, IconSize, UiIconExt};

const SAMPLE_PRESETS: [(&str, &str, Option<StoreKind>); 4] = [
    (
        "SeasFire Global Wildfire (Zarr)",
        "https://s3.bgc-jena.mpg.de:9000/misc/seasfire_rechunked.zarr",
        Some(StoreKind::RemoteZarr),
    ),
    (
        "Sentinel-2 L2A COG (GeoTIFF)",
        "https://sentinel-cogs.s3.us-west-2.amazonaws.com/sentinel-s2-l2a-cogs/36/Q/WD/2020/7/S2A_36QWD_20200701_0_L2A/TCI.tif",
        Some(StoreKind::RemoteGeoTiff),
    ),
    (
        "4D Known-Truth Volume (Procedural)",
        "procedural://volume4d",
        Some(StoreKind::ProceduralVolume4D),
    ),
    (
        "2D Procedural Matrix (Test)",
        "procedural://matrix2d",
        Some(StoreKind::ProceduralRandom),
    ),
];

/// Renders the source intake controls: URL/path text input, format selection,
/// quick sample presets, catalog opener, and drop zone.
pub fn show_source_intake(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Target URL / File Path").strong());

    let has_input = !app.selected.store_target.is_empty();
    let load_icon = if app.is_loading {
        Icon::Hourglass
    } else {
        Icon::DropTray
    };
    let load_text = if app.is_loading { "Loading..." } else { "Load" };

    let mut should_submit = false;
    ui.horizontal(|ui| {
        let edit_width = (ui.available_width() - 70.0).max(100.0);
        let edit_resp = ui.add_sized(
            [edit_width, 22.0],
            egui::TextEdit::singleline(&mut app.selected.store_target)
                .hint_text("https://... or path (.zarr, .nc, .tif)"),
        );

        if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            should_submit = true;
        }

        if ui
            .add_enabled(!app.is_loading, |ui: &mut egui::Ui| {
                ui.icon_button(load_icon, load_text)
            })
            .clicked()
        {
            should_submit = true;
        }
    });

    if should_submit {
        let target = app.selected.store_target.clone();
        app.submit_or_activate_source(&target, Some(app.selected.store_kind));
    }

    ui.add_space(4.0);
    show_format_selector(app, ui);

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.icon_button(Icon::Catalog, "Catalog...").clicked() {
            app.layout.show_catalog_window = true;
        }

        let samples_btn = ui.icon_button(Icon::FolderOpen, "Samples");
        egui::Popup::from_toggle_button_response(&samples_btn)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
            .show(|ui| {
                for (label, url, kind) in SAMPLE_PRESETS {
                    if ui.button(label).clicked() {
                        app.selected.store_target = url.to_string();
                        if let Some(k) = kind {
                            app.selected.store_kind = k;
                        }
                        app.submit_or_activate_source(url, kind);
                    }
                }
            });

        if has_input && ui.button("Clear").clicked() {
            app.selected.store_target.clear();
        }
    });

    ui.add_space(6.0);
    crate::ui::drop_zone::show_drop_zone(ui, None, 54.0);
}

fn show_format_selector(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.icon(Icon::Dataset, IconSize::Xs);
        ui.label(egui::RichText::new("Format:").weak());

        let mut selected = app.selected.store_kind;
        egui::ComboBox::from_id_salt("dataset_store_kind_select")
            .selected_text(match selected {
                StoreKind::RemoteZarr => "Remote Zarr (HTTP/S3)",
                StoreKind::LocalZarr => "Local Zarr (Filesystem)",
                StoreKind::RemoteIcechunk => "Remote Icechunk (HTTP/S3)",
                StoreKind::LocalIcechunk => "Local Icechunk (Filesystem)",
                StoreKind::RemoteGeoTiff => "Remote GeoTIFF / COG",
                StoreKind::LocalGeoTiff => "Local GeoTIFF",
                StoreKind::LocalNetCdf => "Local NetCDF / HDF5",
                StoreKind::ProceduralVolume4D => "4D Known-Truth Volume",
                StoreKind::ProceduralRandom => "2D Procedural Matrix",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut selected,
                    StoreKind::RemoteZarr,
                    "Remote Zarr (HTTP/S3)",
                );
                ui.selectable_value(
                    &mut selected,
                    StoreKind::LocalZarr,
                    "Local Zarr (Filesystem)",
                );
                ui.selectable_value(
                    &mut selected,
                    StoreKind::RemoteIcechunk,
                    "Remote Icechunk (HTTP/S3)",
                );
                ui.selectable_value(
                    &mut selected,
                    StoreKind::LocalIcechunk,
                    "Local Icechunk (Filesystem)",
                );
                ui.selectable_value(
                    &mut selected,
                    StoreKind::RemoteGeoTiff,
                    "Remote GeoTIFF / COG (HTTP/S3)",
                );
                ui.selectable_value(
                    &mut selected,
                    StoreKind::LocalGeoTiff,
                    "Local GeoTIFF (.tif/.tiff/.cog)",
                );
                ui.selectable_value(
                    &mut selected,
                    StoreKind::LocalNetCdf,
                    "Local NetCDF / HDF5 (.nc/.h5)",
                );
                ui.separator();
                ui.selectable_value(
                    &mut selected,
                    StoreKind::ProceduralVolume4D,
                    "4D Known-Truth Volume (Procedural)",
                );
                ui.selectable_value(
                    &mut selected,
                    StoreKind::ProceduralRandom,
                    "2D Procedural Matrix (Procedural)",
                );
            });

        if selected != app.selected.store_kind {
            app.selected.store_kind = selected;
        }
    });
}
