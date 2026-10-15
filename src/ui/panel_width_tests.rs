//! The Dataset, Settings and Dimensions panels keep their width in every language:
//! rows whose translations run longer stack (`flow_row`) instead of
//! widening the panel or spilling past its edge.

use crate::app::OctantApp;
use crate::app::test_support::{make_resident, memory_app, selection_of};
use crate::data::SliceRequest;
use crate::i18n::{DatasetKey, Key, Language, SettingsKey, translate, with_language};
use crate::plots::PlotType;
use crate::ui::dataset::show_dataset_window;
use crate::ui::fonts::FontEngine;
use crate::ui::panel_layout::PANEL_W;
use crate::ui::settings::show_settings_window;
use crate::ui::test_input::Harness;
use crate::ui::variables_panel::show_variable_controls;
use egui::epaint::ClippedShape;
use egui::{Id, Pos2, Rect, Ui, pos2};

pub(super) const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 2400.0));
/// A canvas too narrow for the Dimensions panel's full width beside the
/// Settings panel, so it takes its narrowest.
pub(super) const NARROW: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(420.0, 2400.0));
/// The Dimensions panel's narrowest content width.
pub(super) const DIMENSIONS_MIN_W: f32 = 220.0;
/// Frames for a panel to settle: an area lends its content last frame's
/// size, and the scrolled Settings body follows a frame behind it.
const SETTLE: usize = 8;
/// Rounding of measured against drawn widths.
pub(super) const SLACK: f32 = 1.0;

const PLOT_TYPES: [PlotType; 6] = [
    PlotType::Heatmap,
    PlotType::Line,
    PlotType::Volume,
    PlotType::Sphere,
    PlotType::Surface,
    PlotType::PointCloud,
];

/// `t2m` plotted as `plot_type` from resident blocks, staged in the
/// Dimensions panel, with every optional setting row showing.
fn plotted(plot_type: PlotType) -> OctantApp {
    let (mut app, meta) = memory_app();
    make_resident(&mut app, &SliceRequest::full_range("t2m", &[3, 5, 4]));
    let base = selection_of(&mut app, &meta, "t2m");
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    app.load_selected_variable_block();
    app.layers.base.selection_mut().plot_type = plot_type;
    app.selected.plot_type = plot_type;
    app.layout.show_settings_panel = true;
    app.layout.show_variable_controls = true;
    app.show_coastlines = true;
    app.plot_configs.line.show_points = true;
    app.plot_configs.line.use_custom_color = true;
    app.plot_configs.mesh.sphere_mode = 1;
    let color = &mut app.layers.base.color;
    color.lock_bounds = true;
    color.use_nan_color = true;
    color.use_lowclip = true;
    color.use_highclip = true;
    color.alpha.text = "0:0, 1:1".to_string();
    app
}

/// The center of the painted text `label` in `shapes`, if any.
fn text_center(shapes: &[ClippedShape], label: &str) -> Option<Pos2> {
    shapes.iter().find_map(|clipped| match &clipped.shape {
        egui::Shape::Text(text) if text.galley.text() == label => {
            Some(text.pos + text.galley.size() * 0.5)
        }
        _ => None,
    })
}

/// Width of the area `id` that `run` shows in `lang`, with the app's fonts
/// for it (the CJK fallbacks draw Chinese, Japanese and Korean at their real
/// widths), after opening the collapsed sections titled `open`.
pub(super) fn area_width(
    lang: Language,
    id: &str,
    open: &[Key],
    run: &mut impl FnMut(&mut Ui),
) -> f32 {
    let _fonts = crate::ui::fonts::test_lock();
    with_language(lang, || {
        let mut h = Harness::new(SCREEN);
        FontEngine::on_language_changed(&h.ctx, lang);
        let mut shapes = h.settle(SETTLE, run);
        for &key in open {
            if let Some(pos) = text_center(&shapes, translate(lang, key)) {
                h.click(pos, run);
                shapes = h.settle(SETTLE, run);
            }
        }
        h.area(Id::new(id)).width()
    })
}

/// Horizontal margins of the popup frame both panels draw in.
pub(super) fn frame_margins() -> f32 {
    egui::Frame::popup(&egui::Style::default())
        .total_margin()
        .sum()
        .x
}

#[test]
fn settings_panel_keeps_its_width_in_every_language() {
    let limit = PANEL_W + frame_margins() + SLACK;
    let open = [SettingsKey::Color.into(), SettingsKey::Export.into()];
    let mut wide = Vec::new();
    for plot_type in PLOT_TYPES {
        for &lang in Language::ALL {
            let mut app = plotted(plot_type);
            let mut run = |ui: &mut Ui| show_settings_window(&mut app, ui.ctx(), SCREEN);
            let width = area_width(lang, "octant_settings_area", &open, &mut run);
            if width > limit {
                wide.push(format!("{plot_type:?} {lang:?}: {width:.0}"));
            }
        }
    }
    assert!(
        wide.is_empty(),
        "settings wider than {limit}: {}",
        wide.join(", ")
    );
}

#[test]
fn dimensions_panel_keeps_its_narrowest_width_in_every_language() {
    let limit = DIMENSIONS_MIN_W + frame_margins() + SLACK;
    let mut wide = Vec::new();
    for &lang in Language::ALL {
        let mut app = plotted(PlotType::Heatmap);
        let mut run = |ui: &mut Ui| {
            show_settings_window(&mut app, ui.ctx(), NARROW);
            show_variable_controls(&mut app, ui.ctx(), NARROW);
        };
        let width = area_width(lang, "octant_variables_panel", &[], &mut run);
        if width > limit {
            wide.push(format!("{lang:?}: {width:.0}"));
        }
    }
    assert!(
        wide.is_empty(),
        "dimensions panel wider than {limit}: {}",
        wide.join(", ")
    );
}

/// A 384-well plate (16 x 24) with two long-named acquisitions.
fn wide_plate() -> crate::app::PlateState {
    use crate::data::ome::plate::{PlateAcquisition, PlateWell};
    let rows: Vec<String> = (b'A'..=b'P').map(|c| char::from(c).to_string()).collect();
    let columns: Vec<String> = (1..=24).map(|c| c.to_string()).collect();
    let wells = (0..16)
        .flat_map(|r| (0..24).map(move |c| (r, c)))
        .map(|(row, column)| PlateWell {
            path: format!("{}/{}", rows[row], columns[column]),
            row,
            column,
        })
        .collect();
    let acquisitions = (0..2)
        .map(|id| PlateAcquisition {
            id,
            name: Some(format!("Measurement {id} (2012-07-31_10-41-12)")),
        })
        .collect();
    let plate = crate::data::ome::OmePlate {
        name: Some("A screen with a long plate name to wrap".into()),
        rows,
        columns,
        wells,
        acquisitions,
    };
    crate::app::PlateState::new(
        "https://example.org/plate.zarr",
        crate::app::StoreKind::RemoteZarr,
        std::sync::Arc::new(plate),
    )
}

#[test]
fn dataset_panel_keeps_its_width_in_every_language() {
    let limit = PANEL_W + frame_margins() + SLACK;
    let open = [DatasetKey::MemoryCache.into()];
    let mut wide = Vec::new();
    for (case, loaded, loading) in [
        ("empty", false, false),
        ("loaded", true, false),
        ("loading", true, true),
        ("plate", false, false),
    ] {
        for &lang in Language::ALL {
            let mut app = if loaded {
                plotted(PlotType::Heatmap)
            } else {
                OctantApp::default()
            };
            app.layout.show_dataset_panel = true;
            app.is_loading = loading;
            if case == "plate" {
                app.plate = Some(wide_plate());
            }
            let mut run = |ui: &mut Ui| show_dataset_window(&mut app, ui.ctx(), SCREEN);
            let width = area_width(lang, "octant_dataset_area", &open, &mut run);
            if width > limit {
                wide.push(format!("{case} {lang:?}: {width:.0}"));
            }
        }
    }
    assert!(
        wide.is_empty(),
        "dataset panel wider than {limit}: {}",
        wide.join(", ")
    );
}
