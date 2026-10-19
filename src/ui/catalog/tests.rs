//! The catalog by keyboard: opening it focuses its search field, Down walks
//! the cards and Up returns, Tab stays inside the dialog, and Escape closes
//! it.

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

#[test]
fn down_from_the_search_field_walks_the_cards_and_up_returns() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(SCREEN);
    h.settle(1, &mut run(&mut app));
    app.layout.show_catalog_window = true;
    h.settle(2, &mut run(&mut app));
    let search = focus::last(&h.ctx).expect("the search field has focus");
    let mut press = |key| {
        let focused = h.press(key, Modifiers::NONE, &mut run(&mut app));
        focused.expect("focus stays in the dialog")
    };
    // Down passes the category chips to the first card, then the next one.
    let first = press(Key::ArrowDown);
    let second = press(Key::ArrowDown);
    assert!(
        first.rect.top() > search.rect.bottom() + 40.0,
        "past the chips"
    );
    assert!(second.rect.top() > first.rect.bottom());
    assert_eq!(press(Key::ArrowUp).id, first.id);
    assert_eq!(
        press(Key::ArrowUp).id,
        search.id,
        "back to the search field"
    );
}
