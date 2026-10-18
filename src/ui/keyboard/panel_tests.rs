//! Panels by keyboard: shortcuts open and close them, focus enters a panel
//! opened by keys and returns to its toolbar button when Escape closes it,
//! and a focused grip moves the panel.

use super::{dispatch, focus};
use crate::app::OctantApp;
use crate::ui::panel_layout::Panel;
use crate::ui::test_input::Harness;
use egui::{Key, Modifiers, Rect, Ui, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1400.0, 1000.0));

fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        dispatch(app, ui.ctx(), false);
        crate::ui::top_bar::show_top_bar(app, ui);
        let canvas = ui.available_rect_before_wrap();
        crate::ui::settings::show_settings_window(app, ui.ctx(), canvas);
        focus::end_frame(ui.ctx());
    }
}

fn press(h: &mut Harness, app: &mut OctantApp, key: Key, modifiers: Modifiers) {
    h.key(key, modifiers, &mut run(app));
    h.settle(2, &mut run(app));
}

fn focus_in_settings(h: &Harness) -> bool {
    focus::last(&h.ctx).is_some_and(|f| f.layer.id == Panel::Settings.area_id())
}

#[test]
fn a_shortcut_opens_a_panel_and_escape_closes_it() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(SCREEN);
    h.settle(2, &mut run(&mut app));
    press(&mut h, &mut app, Key::Num4, Modifiers::COMMAND);
    assert!(app.layout.show_settings_panel);
    assert!(focus_in_settings(&h), "focus enters the opened panel");
    press(&mut h, &mut app, Key::Escape, Modifiers::NONE);
    assert!(!app.layout.show_settings_panel);
    press(&mut h, &mut app, Key::Num4, Modifiers::COMMAND);
    press(&mut h, &mut app, Key::Num4, Modifiers::COMMAND);
    assert!(!app.layout.show_settings_panel, "the shortcut toggles");
}

#[test]
fn escape_returns_focus_to_the_button_that_opened_the_panel() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(SCREEN);
    h.settle(2, &mut run(&mut app));
    press(&mut h, &mut app, Key::F10, Modifiers::NONE);
    // Logo, brand label, Dataset, Variables, Dimensions, plot type, Settings.
    for _ in 0..6 {
        press(&mut h, &mut app, Key::ArrowRight, Modifiers::NONE);
    }
    let button = focus::last(&h.ctx).expect("the Settings button");
    press(&mut h, &mut app, Key::Enter, Modifiers::NONE);
    assert!(app.layout.show_settings_panel);
    assert!(focus_in_settings(&h));
    press(&mut h, &mut app, Key::Escape, Modifiers::NONE);
    assert!(!app.layout.show_settings_panel);
    assert_eq!(focus::last(&h.ctx).map(|f| f.id), Some(button.id));
}

#[test]
fn a_focused_grip_moves_its_panel_and_enter_docks_it() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(SCREEN);
    h.settle(2, &mut run(&mut app));
    press(&mut h, &mut app, Key::Num4, Modifiers::COMMAND);
    let docked = h.area(Panel::Settings.area_id()).min;
    press(&mut h, &mut app, Key::ArrowRight, Modifiers::NONE);
    press(&mut h, &mut app, Key::ArrowDown, Modifiers::NONE);
    let moved = h.area(Panel::Settings.area_id()).min;
    assert!(
        moved.x > docked.x && moved.y > docked.y,
        "{docked:?} -> {moved:?}"
    );
    assert!(
        focus_in_settings(&h),
        "the arrows move the panel, not focus"
    );
    press(&mut h, &mut app, Key::Enter, Modifiers::NONE);
    assert_eq!(app.layout.panel_positions.settings, None);
}
