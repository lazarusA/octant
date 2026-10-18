//! Single keys taking focus to the panels and colorbars: D, V, A and S open
//! a panel with focus in it, move focus into an open one, and close it from
//! inside; B goes to the colorbar; the keys work from inside other panels.

use super::focus;
use super::walk_tests::{SCREEN, app, run};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::ui::panel_layout::Panel;
use crate::ui::test_input::Harness;
use egui::{Id, Key, Modifiers};

fn press(h: &mut Harness, app: &mut OctantApp, key: Key) {
    h.key(key, Modifiers::NONE, &mut run(app));
    h.settle(2, &mut run(app));
}

fn focus_in(h: &Harness, area: Id) -> bool {
    focus::last(&h.ctx).is_some_and(|f| f.layer.id == area)
}

#[test]
fn d_opens_the_dataset_panel_at_its_source_field() {
    let mut app = app(false);
    let mut h = Harness::new(SCREEN);
    h.settle(4, &mut run(&mut app));
    press(&mut h, &mut app, Key::D);
    assert!(app.layout.show_dataset_panel);
    assert!(focus_in(&h, Panel::Dataset.area_id()));
    assert!(h.ctx.text_edit_focused(), "ready to type a source");
}

#[test]
fn a_lands_on_plot_data() {
    let mut app = app(false);
    let mut h = Harness::new(SCREEN);
    h.settle(4, &mut run(&mut app));
    press(&mut h, &mut app, Key::A);
    assert!(focus_in(&h, Panel::Dimensions.area_id()));
    press(&mut h, &mut app, Key::Enter);
    assert!(app.layout.show_settings_panel, "Plot Data plotted");
    assert!(!app.layout.show_variable_controls);
}

#[test]
fn keys_jump_between_panels_and_close_from_inside() {
    let mut app = app(false);
    let mut h = Harness::new(SCREEN);
    h.settle(4, &mut run(&mut app));
    press(&mut h, &mut app, Key::S);
    assert!(focus_in(&h, Panel::Settings.area_id()));
    let setting = focus::last(&h.ctx).expect("a setting");
    // From inside Settings, A goes to Dimensions; Settings stays open.
    press(&mut h, &mut app, Key::A);
    assert!(focus_in(&h, Panel::Dimensions.area_id()));
    assert!(app.layout.show_settings_panel);
    // A again, from inside, closes Dimensions and returns to the setting.
    press(&mut h, &mut app, Key::A);
    assert!(!app.layout.show_variable_controls);
    assert_eq!(focus::last(&h.ctx).map(|f| f.id), Some(setting.id));
}

#[test]
fn a_key_moves_focus_into_its_open_panel() {
    let mut app = app(false);
    app.layout.show_settings_panel = true;
    let mut h = Harness::new(SCREEN);
    h.settle(4, &mut run(&mut app));
    press(&mut h, &mut app, Key::S);
    assert!(app.layout.show_settings_panel, "an open panel stays open");
    assert!(focus_in(&h, Panel::Settings.area_id()));
}

#[test]
fn b_goes_to_the_colorbar() {
    let mut app = app(false);
    app.layout.show_hero = false;
    let mut h = Harness::new(SCREEN);
    h.settle(4, &mut run(&mut app));
    press(&mut h, &mut app, Key::B);
    let colorbar = Id::unique(("octant_colorbar_overlay", LayerId::BASE));
    assert!(focus_in(&h, colorbar));
    press(&mut h, &mut app, Key::B);
    assert!(focus_in(&h, colorbar), "the only colorbar stays focused");
}
