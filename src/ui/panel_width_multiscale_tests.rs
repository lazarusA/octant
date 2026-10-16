//! The level note (Dimensions panel) and level select (Settings) of a
//! plotted multiscale variable keep both panels at their width in every
//! language.

use super::panel_width_tests::{
    DIMENSIONS_MIN_W, NARROW, SCREEN, SLACK, area_width, frame_margins,
};
use crate::app::OctantApp;
use crate::app::layers::LevelChoice;
use crate::app::test_support::{poll_until, pyramid_app, selection_of};
use crate::i18n::Language;
use crate::ui::panel_layout::PANEL_W;
use crate::ui::settings::show_settings_window;
use crate::ui::variables_panel::show_variable_controls;
use egui::Ui;

/// `t2m` of the pyramid plotted at a pinned level, staged in both panels.
fn plotted_pyramid() -> OctantApp {
    let (mut app, meta) = pyramid_app();
    app.selected = selection_of(&mut app, &meta, "0/t2m");
    app.selected.level = LevelChoice::Pinned(1);
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    app.layout.show_settings_panel = true;
    app.layout.show_variable_controls = true;
    app
}

#[test]
fn multiscale_rows_keep_both_panels_at_their_width() {
    let settings_limit = PANEL_W + frame_margins() + SLACK;
    let dims_limit = DIMENSIONS_MIN_W + frame_margins() + SLACK;
    let mut wide = Vec::new();
    for &lang in Language::ALL {
        let mut app = plotted_pyramid();
        let mut run = |ui: &mut Ui| show_settings_window(&mut app, ui.ctx(), SCREEN);
        let width = area_width(lang, "octant_settings_area", &[], &mut run);
        if width > settings_limit {
            wide.push(format!("settings {lang:?}: {width:.0}"));
        }
        let mut app = plotted_pyramid();
        let mut run = |ui: &mut Ui| {
            show_settings_window(&mut app, ui.ctx(), NARROW);
            show_variable_controls(&mut app, ui.ctx(), NARROW);
        };
        let width = area_width(lang, "octant_variables_panel", &[], &mut run);
        if width > dims_limit {
            wide.push(format!("dimensions {lang:?}: {width:.0}"));
        }
    }
    assert!(
        wide.is_empty(),
        "wider than their panels: {}",
        wide.join(", ")
    );
}
