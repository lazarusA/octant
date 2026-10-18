//! Plot types offered for a variable, and the centered picker by keyboard:
//! a number key picks, the arrows and Enter pick, Escape leaves it alone.

use super::options::{Blocked, ORDER, plot_options};
use super::picker::show_plot_type_picker;
use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::keyboard::dispatch;
use crate::ui::panel_width_tests::plotted;
use crate::ui::test_input::Harness;
use egui::{Key, Modifiers, Rect, Ui, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 900.0));

#[test]
fn without_a_3d_variable_volumes_are_blocked() {
    let mut app = OctantApp::default();
    let options = plot_options(&mut app);
    assert_eq!(options.map(|o| o.plot_type), ORDER);
    let blocked = |t: PlotType| {
        options
            .iter()
            .find(|o| o.plot_type == t)
            .and_then(|o| o.blocked)
    };
    assert_eq!(blocked(PlotType::Heatmap), None);
    assert_eq!(blocked(PlotType::Line), None);
    assert_eq!(blocked(PlotType::Volume), Some(Blocked::Requires3D));
    assert_eq!(blocked(PlotType::PointCloud), Some(Blocked::Requires3D));
}

fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        dispatch(app, ui.ctx(), true);
        show_plot_type_picker(app, ui.ctx());
    }
}

fn press(h: &mut Harness, app: &mut OctantApp, key: Key) {
    h.press(key, Modifiers::NONE, &mut run(app));
}

#[test]
fn p_opens_the_picker_and_a_number_key_picks() {
    let mut app = plotted(PlotType::Heatmap);
    let mut h = Harness::new(SCREEN);
    h.settle(2, &mut run(&mut app));
    press(&mut h, &mut app, Key::P);
    assert!(app.layout.show_plot_picker);
    press(&mut h, &mut app, Key::Num2);
    assert_eq!(app.selected.plot_type, PlotType::Line);
    assert!(!app.layout.show_plot_picker, "picking closes it");
}

#[test]
fn arrows_and_enter_pick_and_escape_changes_nothing() {
    let mut app = plotted(PlotType::Heatmap);
    let mut h = Harness::new(SCREEN);
    h.settle(2, &mut run(&mut app));
    press(&mut h, &mut app, Key::P);
    press(&mut h, &mut app, Key::Escape);
    assert!(!app.layout.show_plot_picker);
    assert_eq!(app.selected.plot_type, PlotType::Heatmap);

    // Focus starts on the current type; Right moves to the next tile.
    press(&mut h, &mut app, Key::P);
    press(&mut h, &mut app, Key::ArrowRight);
    press(&mut h, &mut app, Key::Enter);
    assert_eq!(app.selected.plot_type, PlotType::Line);
    assert!(!app.layout.show_plot_picker);
}

#[test]
fn a_click_outside_closes_the_picker() {
    let mut app = plotted(PlotType::Heatmap);
    let mut h = Harness::new(SCREEN);
    h.settle(2, &mut run(&mut app));
    press(&mut h, &mut app, Key::P);
    h.click(pos2(5.0, 5.0), &mut run(&mut app));
    assert!(!app.layout.show_plot_picker);
    assert_eq!(app.selected.plot_type, PlotType::Heatmap);
}

#[test]
fn the_picker_keeps_its_width_in_every_language() {
    let make_app = || {
        let mut app = plotted(PlotType::Heatmap);
        app.layout.show_plot_picker = true;
        app
    };
    let width = super::picker::WIDTH;
    crate::ui::panel_width_tests::assert_window_width(
        "plot_type_picker",
        width,
        make_app,
        show_plot_type_picker,
    );
}
