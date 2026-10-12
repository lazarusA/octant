use crate::app::OctantApp;
use crate::app::StoreKind;
use crate::i18n::{CommonKey, DatasetKey, tr};
use crate::ui::icons::{Icon, IconSize, UiIconExt};

const SAMPLE_PRESETS: [(DatasetKey, &str, Option<StoreKind>); 4] = [
    (
        DatasetKey::SampleSeasFire,
        "https://s3.bgc-jena.mpg.de:9000/misc/seasfire_rechunked.zarr",
        Some(StoreKind::RemoteZarr),
    ),
    (
        DatasetKey::SampleSentinel2,
        "https://sentinel-cogs.s3.us-west-2.amazonaws.com/sentinel-s2-l2a-cogs/36/Q/WD/2020/7/S2A_36QWD_20200701_0_L2A/TCI.tif",
        Some(StoreKind::RemoteGeoTiff),
    ),
    (
        DatasetKey::FormatVolume4D,
        "procedural://volume4d",
        Some(StoreKind::ProceduralVolume4D),
    ),
    (
        DatasetKey::SampleMatrixTest,
        "procedural://matrix2d",
        Some(StoreKind::ProceduralRandom),
    ),
];

const FORMAT_OPTIONS: [(StoreKind, DatasetKey); 9] = [
    (StoreKind::RemoteZarr, DatasetKey::FormatRemoteZarr),
    (StoreKind::LocalZarr, DatasetKey::FormatLocalZarr),
    (StoreKind::RemoteIcechunk, DatasetKey::FormatRemoteIcechunk),
    (StoreKind::LocalIcechunk, DatasetKey::FormatLocalIcechunk),
    (StoreKind::RemoteGeoTiff, DatasetKey::FormatRemoteGeoTiff),
    (StoreKind::LocalGeoTiff, DatasetKey::FormatLocalGeoTiff),
    (StoreKind::LocalNetCdf, DatasetKey::FormatLocalNetCdf),
    (StoreKind::ProceduralVolume4D, DatasetKey::FormatVolume4D),
    (StoreKind::ProceduralRandom, DatasetKey::FormatMatrix2D),
];

/// Renders the source intake controls: URL/path text input, format selection,
/// quick sample presets, catalog opener, and drop zone.
pub fn show_source_intake(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new(tr(DatasetKey::SourceUrlOrPath)).strong());

    let has_input = !app.selected.store_target.is_empty();
    if show_input_bar(app, ui) {
        let target = app.selected.store_target.clone();
        app.submit_or_activate_source(&target, Some(app.selected.store_kind));
    }

    ui.add_space(4.0);
    show_format_selector(app, ui);

    ui.add_space(4.0);
    show_intake_actions(app, ui, has_input);

    ui.add_space(6.0);
    crate::ui::drop_zone::show_drop_zone(ui, None, 54.0);
}

fn show_input_bar(app: &mut OctantApp, ui: &mut egui::Ui) -> bool {
    let load_icon = if app.is_loading {
        Icon::Hourglass
    } else {
        Icon::DropTray
    };
    let load_text = if app.is_loading {
        tr(CommonKey::Loading)
    } else {
        tr(DatasetKey::Load)
    };
    let mut submit = false;

    ui.horizontal(|ui| {
        let edit_width = (ui.available_width() - 70.0).max(100.0);
        let edit_resp = ui.add_sized(
            [edit_width, 22.0],
            egui::TextEdit::singleline(&mut app.selected.store_target)
                .hint_text(tr(DatasetKey::TargetPathHint)),
        );

        if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            submit = true;
        }

        if ui
            .add_enabled(!app.is_loading, |ui: &mut egui::Ui| {
                ui.icon_button(load_icon, load_text)
            })
            .clicked()
        {
            submit = true;
        }
    });

    submit
}

fn show_intake_actions(app: &mut OctantApp, ui: &mut egui::Ui, has_input: bool) {
    ui.horizontal(|ui| {
        if ui
            .icon_button(Icon::Catalog, tr(DatasetKey::CatalogEllipsis))
            .clicked()
        {
            app.layout.show_catalog_window = true;
        }

        let samples_btn = ui.icon_button(Icon::FolderOpen, tr(DatasetKey::SampleDatasets));
        egui::Popup::from_toggle_button_response(&samples_btn)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
            .show(|ui| {
                for (label, url, kind) in SAMPLE_PRESETS {
                    if ui.button(tr(label)).clicked() {
                        app.selected.store_target = url.to_string();
                        if let Some(k) = kind {
                            app.selected.store_kind = k;
                        }
                        app.submit_or_activate_source(url, kind);
                    }
                }
            });

        if has_input && ui.button(tr(DatasetKey::Clear)).clicked() {
            app.selected.store_target.clear();
        }
    });
}

fn format_label(kind: StoreKind) -> DatasetKey {
    match kind {
        StoreKind::RemoteZarr => DatasetKey::FormatRemoteZarr,
        StoreKind::LocalZarr => DatasetKey::FormatLocalZarr,
        StoreKind::RemoteIcechunk => DatasetKey::FormatRemoteIcechunk,
        StoreKind::LocalIcechunk => DatasetKey::FormatLocalIcechunk,
        StoreKind::RemoteGeoTiff => DatasetKey::FormatRemoteGeoTiffShort,
        StoreKind::LocalGeoTiff => DatasetKey::FormatLocalGeoTiffShort,
        StoreKind::LocalNetCdf => DatasetKey::FormatLocalNetCdfShort,
        StoreKind::ProceduralVolume4D => DatasetKey::FormatVolume4DShort,
        StoreKind::ProceduralRandom => DatasetKey::FormatMatrix2DShort,
    }
}

fn show_format_selector(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.icon(Icon::Dataset, IconSize::Xs);
        ui.label(egui::RichText::new(tr(DatasetKey::FormatLabel)).weak());

        let mut selected = app.selected.store_kind;
        egui::ComboBox::from_id_salt("dataset_store_kind_select")
            .selected_text(tr(format_label(selected)))
            .show_ui(ui, |ui| {
                for (i, (kind, label)) in FORMAT_OPTIONS.iter().enumerate() {
                    if i == 7 {
                        ui.separator();
                    }
                    ui.selectable_value(&mut selected, *kind, tr(*label));
                }
            });

        if selected != app.selected.store_kind {
            app.selected.store_kind = selected;
        }
    });
}
