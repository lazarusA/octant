//! Picker filtering tests.

use super::search::{FamilyFilter, FilterKey, filter_ids};
use crate::utils::colormap::{ColormapKind, builtin, registry};

#[test]
fn empty_filter_lists_every_builtin() {
    let _registry = crate::utils::colormap::registry::test_lock();
    let ids = filter_ids(&FilterKey::default());
    assert!(ids.len() >= builtin().maps.len());
}

#[test]
fn query_matches_name_case_insensitively() {
    let _registry = crate::utils::colormap::registry::test_lock();
    let key = FilterKey {
        query: "VIRIDIS".into(),
        ..Default::default()
    };
    let ids = filter_ids(&key);
    let found = ids
        .iter()
        .any(|&id| registry::with_entry(id, |e| e.key == "matplotlib:viridis").unwrap_or(false));
    assert!(found);
}

#[test]
fn query_matches_family_name() {
    let _registry = crate::utils::colormap::registry::test_lock();
    let key = FilterKey {
        query: "cmocean".into(),
        ..Default::default()
    };
    let ids = filter_ids(&key);
    assert!(ids.len() >= 20);
}

#[test]
fn kind_and_family_filters_compose() {
    let _registry = crate::utils::colormap::registry::test_lock();
    let Some(cmocean) = builtin().families.iter().position(|f| f.key == "cmocean") else {
        panic!("cmocean family missing");
    };
    let key = FilterKey {
        kind: Some(ColormapKind::Diverging),
        family: Some(FamilyFilter::Builtin(cmocean)),
        ..Default::default()
    };
    for id in filter_ids(&key) {
        let ok = registry::with_entry(id, |e| {
            e.kind == ColormapKind::Diverging && e.family == Some(cmocean)
        });
        assert_eq!(ok, Some(true));
    }
    assert!(!filter_ids(&key).is_empty());
}

#[test]
fn row_names_are_elided_and_cached() {
    let _registry = registry::test_lock();
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let mut names = super::label::NameCache::default();
        let painter = ui.painter();
        let long =
            registry::find("colorcet:linear_protanopic_deuteranopic_kbjyw_5_95_c25").unwrap_or(0);
        let Some(cut) = names.get(painter, long, super::label::ROW_FONT_SIZE, 80.0) else {
            panic!("missing galley")
        };
        assert!(cut.elided);
        assert!(cut.size().x <= 80.5);
        let Some(again) = names.get(painter, long, super::label::ROW_FONT_SIZE, 80.0) else {
            panic!("missing galley")
        };
        assert!(
            std::sync::Arc::ptr_eq(&cut, &again),
            "same width reuses the galley"
        );
        let Some(wider) = names.get(painter, long, super::label::ROW_FONT_SIZE, 400.0) else {
            panic!("missing galley")
        };
        assert!(!wider.elided, "a new width lays out again");
    });
    // Headless frame: font atlas updates are intentionally not uploaded.
    output.textures_delta.clear();
}

#[test]
fn panel_fits_its_height_with_the_editor_open_or_closed() {
    let _registry = registry::test_lock();
    let ctx = egui::Context::default();
    let mut app = crate::app::OctantApp::default();
    for editor_open in [false, true] {
        for max_height in [500.0_f32, 600.0, 700.0, 900.0] {
            app.colormaps.picker.below_list_height = 0.0;
            let mut last_total = 0.0;
            // A few frames: the editor's open animation and the height measured
            // below the list settle after the first frames.
            for frame in 0..20 {
                let input = egui::RawInput {
                    time: Some(f64::from(frame) * 0.1),
                    ..Default::default()
                };
                let mut output = ctx.run_ui(input, |ui| {
                    let id = super::editor::section_id(ui);
                    let mut state =
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            id,
                            editor_open,
                        );
                    state.set_open(editor_open);
                    state.store(ui.ctx());
                    let start = ui.cursor().min.y;
                    super::render_colormap_contents(&mut app, ui, max_height);
                    last_total = ui.cursor().min.y - start;
                });
                output.textures_delta.clear();
            }
            assert!(
                last_total <= max_height,
                "editor open: {editor_open}, height {max_height}: content is {last_total}"
            );
        }
    }
}
