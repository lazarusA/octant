//! Moving between the landing screen and the plot by keyboard, and plotting
//! the staged variable without reaching for Plot Data.

use super::dispatch;
use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::panel_width_tests::plotted;
use crate::ui::test_input::Harness;
use egui::{Key, Modifiers, Rect, Ui, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));

fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        let plot_shown = !app.hero_shown();
        dispatch(app, ui.ctx(), plot_shown);
    }
}

#[test]
fn h_toggles_the_landing_screen_and_escape_leaves_it() {
    let mut app = plotted(PlotType::Heatmap);
    assert!(app.has_plot());
    app.layout.show_hero = false;
    let mut h = Harness::new(SCREEN);
    h.key(Key::H, Modifiers::NONE, &mut run(&mut app));
    assert!(app.hero_shown());
    h.key(Key::H, Modifiers::NONE, &mut run(&mut app));
    assert!(!app.hero_shown());
    h.key(Key::H, Modifiers::NONE, &mut run(&mut app));
    h.key(Key::Escape, Modifiers::NONE, &mut run(&mut app));
    assert!(!app.hero_shown(), "Escape goes back to the plot");
}

#[test]
fn escape_stays_on_the_landing_screen_without_a_plot() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(SCREEN);
    h.key(Key::Escape, Modifiers::NONE, &mut run(&mut app));
    assert!(app.layout.show_hero);
}

#[test]
fn cmd_enter_plots_the_staged_variable() {
    let mut app = plotted(PlotType::Heatmap);
    app.layout.show_hero = true;
    app.layout.show_variable_controls = true;
    app.layout.show_settings_panel = false;
    let mut h = Harness::new(SCREEN);
    h.key(Key::Enter, Modifiers::COMMAND, &mut run(&mut app));
    assert!(!app.layout.show_hero, "plotting leaves the landing screen");
    assert!(app.layout.show_settings_panel, "as Plot Data does");
}
