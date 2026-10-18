//! A colorbar by keyboard: focus on its bar reveals its controls, and Enter
//! opens the colormap picker, whose list the arrows walk (Up on its first
//! row back to the search field) and Enter picks.

use super::controls_tests::{circles, harness, run};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::ui::test_input::Harness;
use egui::pos2;

#[test]
fn keyboard_focus_on_the_bar_reveals_the_controls_and_picks_a_colormap() {
    use crate::ui::keyboard::focus;
    use egui::{Key, Modifiers};

    let mut app = OctantApp::default();
    let mut h = harness(&mut app);
    h.pointer(pos2(5.0, 5.0), None, &mut run(&mut app));
    focus::request(&h.ctx, super::bar_id(LayerId::BASE));
    let shapes = h.settle(2, &mut run(&mut app));
    assert!(circles(&shapes) >= 6, "focus in the panel shows the grip");

    let press = |h: &mut Harness, app: &mut OctantApp, key| {
        h.press(key, Modifiers::NONE, &mut run(app));
    };
    press(&mut h, &mut app, Key::Enter);
    assert!(egui::Popup::is_any_open(&h.ctx), "Enter opens the picker");
    // The search field has focus: Down enters the list, Down moves on.
    press(&mut h, &mut app, Key::ArrowDown);
    press(&mut h, &mut app, Key::ArrowDown);
    let picked = app.colormaps.picker.focused_row.expect("a row is focused");
    // Up on the first row returns to the search field, Down comes back.
    press(&mut h, &mut app, Key::ArrowUp);
    press(&mut h, &mut app, Key::ArrowUp);
    assert_eq!(app.colormaps.picker.focused_row, None, "the search field");
    press(&mut h, &mut app, Key::ArrowDown);
    press(&mut h, &mut app, Key::ArrowDown);
    assert_eq!(app.colormaps.picker.focused_row, Some(picked));
    press(&mut h, &mut app, Key::Enter);
    assert_eq!(app.layers.base.color.colormap, picked);
}
