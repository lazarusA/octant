//! The gallery by keyboard: opening it focuses its search field, Down walks
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
        super::show_gallery_window(app, ui.ctx());
        focus::end_frame(ui.ctx());
    }
}

fn in_dialog(h: &Harness) -> bool {
    focus::last(&h.ctx).is_some_and(|f| f.layer.id == Id::unique("gallery_modal_dialog"))
}

#[test]
fn the_gallery_keeps_focus_until_escape_closes_it() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(SCREEN);
    h.settle(1, &mut run(&mut app));
    app.layout.show_gallery_window = true;
    h.settle(2, &mut run(&mut app));
    assert!(in_dialog(&h), "the search field takes focus");
    // Past the last widget egui holds no focus for one Tab, then wraps.
    let mut inside = 0;
    for _ in 0..40 {
        h.key(Key::Tab, Modifiers::NONE, &mut run(&mut app));
        let wrapping = focus::last(&h.ctx).is_none();
        assert!(wrapping || in_dialog(&h), "Tab stays in the dialog");
        inside += usize::from(!wrapping);
    }
    assert!(inside > 30, "Tab wraps back into the dialog");
    h.key(Key::Escape, Modifiers::NONE, &mut run(&mut app));
    assert!(!app.layout.show_gallery_window);
}

#[test]
fn down_from_the_search_field_walks_the_cards_and_up_returns() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(SCREEN);
    h.settle(1, &mut run(&mut app));
    app.layout.show_gallery_window = true;
    h.settle(2, &mut run(&mut app));
    let search = focus::last(&h.ctx).expect("the search field has focus");
    let mut press = |key| {
        let focused = h.press(key, Modifiers::NONE, &mut run(&mut app));
        focused.expect("focus stays in the dialog")
    };
    // Down passes the chips to the first card, then the next one in
    // reading order (beside it in the grid, or on the next row).
    let first = press(Key::ArrowDown);
    let second = press(Key::ArrowDown);
    assert!(
        first.rect.top() > search.rect.bottom() + 40.0,
        "past the chips"
    );
    let beside = second.rect.left() > first.rect.right();
    assert!(beside || second.rect.top() > first.rect.bottom());
    assert_eq!(press(Key::ArrowUp).id, first.id);
    assert_eq!(
        press(Key::ArrowUp).id,
        search.id,
        "back to the search field"
    );
}

#[test]
fn cards_stay_inside_the_dialog_in_rows() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(SCREEN);
    h.settle(1, &mut run(&mut app));
    app.layout.show_gallery_window = true;
    h.settle(2, &mut run(&mut app));
    let content = focus::last(&h.ctx)
        .expect("the search field has focus")
        .rect;
    let mut cards = Vec::new();
    for _ in 0..30 {
        h.key(Key::Tab, Modifiers::NONE, &mut run(&mut app));
        // Cards are the tall widgets; the rest are chips and buttons.
        let tall = focus::last(&h.ctx)
            .map(|f| f.rect)
            .filter(|r| r.height() > 100.0);
        cards.extend(tall.filter(|r| !cards.contains(r)));
    }
    assert!(cards.len() >= 4, "{cards:?}");
    for card in &cards {
        assert!(card.right() <= content.right(), "{card:?} past {content:?}");
    }
    // A row's cards share their top and sit the grid's gap apart.
    let row: Vec<_> = cards.iter().filter(|c| c.top() == cards[0].top()).collect();
    for pair in row.windows(2) {
        assert_eq!(
            pair[1].left() - pair[0].right(),
            super::card::GAP,
            "{row:?}"
        );
    }
}
