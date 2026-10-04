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
fn row_names_are_elided_to_fit() {
    let _registry = registry::test_lock();
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let painter = ui.painter();
        let long =
            registry::find("colorcet:linear_protanopic_deuteranopic_kbjyw_5_95_c25").unwrap_or(0);
        let name =
            |width| super::label::name_galley(painter, long, super::label::ROW_FONT_SIZE, width);
        let Some(cut) = name(80.0) else {
            panic!("missing galley")
        };
        assert!(cut.elided);
        assert!(cut.size().x <= 80.5);
        let Some(again) = name(80.0) else {
            panic!("missing galley")
        };
        assert!(
            std::sync::Arc::ptr_eq(&cut, &again),
            "egui's galley cache reuses the layout within a frame"
        );
        let Some(wider) = name(400.0) else {
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

/// Heights of a popup-like `Area` (which sizes its content from last frame)
/// holding the panel's scroll area, over frames whose content is `contents` tall.
fn panel_heights(contents: &[f32], max_height: f32) -> Vec<f32> {
    let ctx = egui::Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 800.0));
    let mut heights = Vec::new();
    for &content in contents {
        let input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            let area = egui::Area::new(egui::Id::new("colormap_panel_test"))
                .fixed_pos(egui::Pos2::ZERO)
                .show(ui.ctx(), |ui| {
                    super::panel_scroll_area(max_height).show(ui, |ui| {
                        ui.allocate_exact_size(egui::vec2(100.0, content), egui::Sense::hover());
                    });
                });
            heights.push(area.response.rect.height());
        });
        output.textures_delta.clear();
    }
    heights
}

#[test]
fn panel_grows_with_its_content_instead_of_scrolling() {
    let heights = panel_heights(&[100.0, 300.0, 300.0, 300.0], 600.0);
    assert_eq!(heights.last().copied(), Some(300.0), "heights: {heights:?}");
}

#[test]
fn panel_scrolls_only_past_the_space_below_the_button() {
    let heights = panel_heights(&[100.0, 900.0, 900.0, 900.0], 600.0);
    assert_eq!(heights.last().copied(), Some(600.0), "heights: {heights:?}");
}

/// Atlas texel corners of every glyph of `galley`.
fn glyph_uvs(galley: Option<&std::sync::Arc<egui::Galley>>) -> Vec<[[u16; 2]; 2]> {
    galley
        .into_iter()
        .flat_map(|g| g.rows.iter())
        .flat_map(|r| r.row.glyphs.iter().map(|g| [g.uv_rect.min, g.uv_rect.max]))
        .collect()
}

/// Switching theme changes egui's text options, which rebuilds the fonts and
/// their glyph atlas: names laid out before must not keep stale atlas positions.
#[test]
fn colormap_names_follow_the_font_atlas_across_theme_switches() {
    let _registry = registry::test_lock();
    let ctx = egui::Context::default();
    let id = registry::default_id();
    for theme in [egui::Theme::Dark, egui::Theme::Light, egui::Theme::Dark] {
        ctx.set_theme(theme);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let painter = ui.painter();
            // Other text first, as in the app, so the rebuilt atlas fills differently.
            let font = egui::FontId::proportional(super::label::ROW_FONT_SIZE);
            let _ = painter.layout_no_wrap(
                format!("{theme:?} unrelated 0123456789 qwxz"),
                font.clone(),
                egui::Color32::PLACEHOLDER,
            );
            let shown = super::label::name_galley(painter, id, super::label::ROW_FONT_SIZE, 200.0);
            let fresh = registry::with_entry(id, |e| {
                crate::ui::hover::card::layout::line(
                    painter,
                    &e.name,
                    font,
                    egui::Color32::PLACEHOLDER,
                    200.0,
                )
            });
            assert_eq!(
                glyph_uvs(shown.as_ref()),
                glyph_uvs(fresh.as_ref()),
                "{theme:?}"
            );
        });
        output.textures_delta.clear();
    }
}
