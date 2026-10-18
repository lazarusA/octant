//! The catalog by keyboard: opening it focuses its search field, Tab stays
//! inside the dialog, and Escape closes it.

use crate::app::OctantApp;
use crate::ui::keyboard::focus;
use crate::ui::test_input::Harness;
use egui::{Id, Key, Modifiers, Rect, Ui, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 900.0));

fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        // A widget behind the dialog, which Tab must not reach.
        let _ = ui.button("behind");
        super::show_catalog_window(app, ui.ctx());
        focus::end_frame(ui.ctx());
    }
}

fn in_dialog(h: &Harness) -> bool {
    focus::last(&h.ctx).is_some_and(|f| f.layer.id == Id::unique("catalog_modal_dialog"))
}

#[test]
fn the_catalog_keeps_focus_until_escape_closes_it() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(SCREEN);
    h.settle(1, &mut run(&mut app));
    app.layout.show_catalog_window = true;
    h.settle(2, &mut run(&mut app));
    assert!(in_dialog(&h), "the search field takes focus");
    for _ in 0..40 {
        h.key(Key::Tab, Modifiers::NONE, &mut run(&mut app));
        assert!(in_dialog(&h), "Tab stays in the dialog");
    }
    h.key(Key::Escape, Modifiers::NONE, &mut run(&mut app));
    assert!(!app.layout.show_catalog_window);
}
