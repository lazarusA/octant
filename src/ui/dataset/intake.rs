use crate::app::OctantApp;
use crate::app::StoreKind;
use crate::i18n::{CommonKey, DatasetKey, tr};
use crate::ui::flow_row;
use crate::ui::icons::{Icon, IconSize, UiIconExt, icon_button_width};

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

pub(super) const FORMAT_OPTIONS: [(StoreKind, DatasetKey); 9] = [
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

/// Narrowest the source field gets beside the Load button; any narrower and
/// the button moves below it.
const MIN_FIELD_W: f32 = 100.0;

/// Width of the format select, whatever format it shows.
const FORMAT_SELECT_W: f32 = 160.0;

/// The source field and the Load button; whether either submitted it.
fn show_input_bar(app: &mut OctantApp, ui: &mut egui::Ui) -> bool {
    let (load, loading) = (tr(DatasetKey::Load), tr(CommonKey::Loading));
    let [load_w, loading_w] = flow_row::cached_widths(ui, "dataset_load", (load, loading), |ui| {
        [icon_button_width(ui, load), icon_button_width(ui, loading)]
    });
    // Room for the button at its widest label, so the field keeps its width
    // while loading.
    let button_w = load_w.max(loading_w);
    let target = &mut app.selected.store_target;
    if flow_row::fits(ui, &[MIN_FIELD_W, button_w]) {
        ui.horizontal(|ui| {
            let spacing = ui.spacing().item_spacing.x;
            let field_w = ui.available_width() - button_w - spacing;
            let entered = source_field(ui, target, field_w);
            load_button(ui, app.is_loading) || entered
        })
        .inner
    } else {
        let entered = source_field(ui, target, ui.available_width());
        let mut clicked = false;
        flow_row::right_aligned(ui, &[button_w], |ui, _| {
            clicked = load_button(ui, app.is_loading);
        });
        entered || clicked
    }
}

/// The source field, `width` wide; whether Enter submitted it.
fn source_field(ui: &mut egui::Ui, target: &mut String, width: f32) -> bool {
    let edit = egui::TextEdit::singleline(target).hint_text(tr(DatasetKey::TargetPathHint));
    let response = ui.add_sized([width, 22.0], edit);
    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
}

/// The Load button, an hourglass while `loading`; whether it was clicked.
fn load_button(ui: &mut egui::Ui, loading: bool) -> bool {
    let (icon, text) = if loading {
        (Icon::Hourglass, tr(CommonKey::Loading))
    } else {
        (Icon::DropTray, tr(DatasetKey::Load))
    };
    ui.add_enabled(!loading, |ui: &mut egui::Ui| ui.icon_button(icon, text))
        .clicked()
}

fn show_intake_actions(app: &mut OctantApp, ui: &mut egui::Ui, has_input: bool) {
    ui.horizontal_wrapped(|ui| {
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

/// The format select: its selected name is cut to `FORMAT_SELECT_W` (the
/// full name on hover) so a long one never widens the panel; the list opens
/// at full width.
pub(super) fn show_format_selector(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.icon(Icon::Dataset, IconSize::Xs);
        ui.label(egui::RichText::new(tr(DatasetKey::FormatLabel)).weak());

        let mut selected = app.selected.store_kind;
        let name = tr(format_label(selected));
        let size = egui::vec2(FORMAT_SELECT_W, ui.spacing().interact_size.y);
        ui.allocate_ui(size, |ui| {
            egui::ComboBox::from_id_salt("dataset_store_kind_select")
                .width(FORMAT_SELECT_W)
                .truncate()
                .selected_text(name)
                .show_ui(ui, |ui| {
                    for (i, (kind, label)) in FORMAT_OPTIONS.iter().enumerate() {
                        if i == 7 {
                            ui.separator();
                        }
                        ui.selectable_value(&mut selected, *kind, tr(*label));
                    }
                })
                .response
                .on_hover_text(name);
        });

        if selected != app.selected.store_kind {
            app.selected.store_kind = selected;
        }
    });
}
