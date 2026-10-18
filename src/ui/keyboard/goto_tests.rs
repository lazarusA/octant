//! Single keys taking focus to the panels and colorbars: D, V, A and S open
//! a panel with focus in it, move focus into an open one, and close it from
//! inside; B goes to the colorbar; the keys work from inside other panels;
//! a focused grip moves its panel.

use super::test_support::{press, window};
use crate::app::layers::LayerId;
use crate::ui::panel_layout::Panel;
use egui::{Id, Key, Modifiers};

const NONE: Modifiers = Modifiers::NONE;

#[test]
fn d_opens_the_dataset_panel_at_its_source_field() {
    let (mut app, mut h) = window(false);
    press(&mut h, &mut app, Key::D, NONE);
    assert!(h.focus_in(Panel::Dataset.area_id()));
    assert!(h.ctx.text_edit_focused(), "ready to type a source");
}

#[test]
fn a_lands_on_plot_data() {
    let (mut app, mut h) = window(false);
    press(&mut h, &mut app, Key::A, NONE);
    assert!(h.focus_in(Panel::Dimensions.area_id()));
    press(&mut h, &mut app, Key::Enter, NONE);
    assert!(app.layout.show_settings_panel, "Plot Data plotted");
    assert!(!app.layout.show_variable_controls);
}

#[test]
fn keys_enter_jump_between_and_close_panels() {
    let (mut app, mut h) = window(false);
    app.layout.show_settings_panel = true;
    let setting = press(&mut h, &mut app, Key::S, NONE).map(|f| f.id);
    assert!(
        h.focus_in(Panel::Settings.area_id()),
        "S enters the open panel"
    );
    // From inside Settings, A goes to Dimensions; Settings stays open.
    press(&mut h, &mut app, Key::A, NONE);
    assert!(h.focus_in(Panel::Dimensions.area_id()));
    assert!(app.layout.show_settings_panel);
    // A again, from inside, closes Dimensions and returns to the setting.
    let back = press(&mut h, &mut app, Key::A, NONE).map(|f| f.id);
    assert!(!app.layout.show_variable_controls);
    assert_eq!(back, setting);
}

#[test]
fn b_goes_to_the_colorbar() {
    let (mut app, mut h) = window(false);
    press(&mut h, &mut app, Key::B, NONE);
    assert!(h.focus_in(Id::unique(("octant_colorbar_overlay", LayerId::BASE))));
}

#[test]
fn a_focused_grip_moves_its_panel_and_enter_docks_it() {
    let (mut app, mut h) = window(false);
    press(&mut h, &mut app, Key::S, NONE);
    // Focus enters at the first setting; the grip is just before it.
    press(&mut h, &mut app, Key::Tab, Modifiers::SHIFT);
    let docked = h.area(Panel::Settings.area_id()).min;
    press(&mut h, &mut app, Key::ArrowRight, NONE);
    press(&mut h, &mut app, Key::ArrowDown, NONE);
    let moved = h.area(Panel::Settings.area_id()).min;
    assert!(
        moved.x > docked.x && moved.y > docked.y,
        "{docked:?} -> {moved:?}"
    );
    assert!(
        h.focus_in(Panel::Settings.area_id()),
        "the arrows move the panel, not focus"
    );
    press(&mut h, &mut app, Key::Enter, NONE);
    assert_eq!(app.layout.panel_positions.settings, None);
}
