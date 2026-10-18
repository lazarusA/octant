//! The keymap's matching, focus visibility, the focus kept across frames,
//! and region jumps.

use super::focus;
use super::keymap::{Action, Keys, Shortcut, TABLE, take};
use crate::ui::test_input::Harness;
use egui::{Key, Modifiers, Pos2, Rect, Ui, Vec2, pos2, vec2};

fn shortcut(action: Action) -> &'static Shortcut {
    TABLE
        .iter()
        .find(|s| s.action == action)
        .expect("action is bound")
}

fn harness() -> Harness {
    Harness::new(Rect::from_min_size(Pos2::ZERO, vec2(200.0, 200.0)))
}

/// Which of `actions` one frame pressing `key` with `modifiers` triggers,
/// taken in table order as the dispatcher does.
fn triggered(key: Key, modifiers: Modifiers, actions: &[Action]) -> Vec<Action> {
    let mut hits = Vec::new();
    harness().key(key, modifiers, &mut |ui| {
        for s in TABLE.iter().filter(|s| actions.contains(&s.action)) {
            if take(ui.ctx(), s).is_some() {
                hits.push(s.action);
            }
        }
    });
    hits
}

#[test]
fn modifiers_must_match_exactly() {
    let save = [Action::QuickSave, Action::ExportDialog];
    assert_eq!(
        triggered(Key::S, Modifiers::COMMAND, &save),
        [Action::QuickSave]
    );
    let cmd_shift = Modifiers::COMMAND.plus(Modifiers::SHIFT);
    assert_eq!(triggered(Key::S, cmd_shift, &save), [Action::ExportDialog]);
    assert!(triggered(Key::S, Modifiers::NONE, &save).is_empty());
    assert!(triggered(Key::C, Modifiers::SHIFT, &[Action::ToggleCrop]).is_empty());
}

#[test]
fn a_press_is_taken_once() {
    let crop = shortcut(Action::ToggleCrop);
    harness().key(Key::C, Modifiers::NONE, &mut |ui| {
        assert_eq!(take(ui.ctx(), crop), Some(Vec2::ZERO));
        assert_eq!(take(ui.ctx(), crop), None);
        assert!(
            !ui.input(|i| i.key_pressed(Key::C)),
            "no widget reads it after"
        );
    });
}

#[test]
fn arrows_read_as_directions() {
    let arrows = Shortcut {
        keys: Keys::Arrows,
        ..*shortcut(Action::ToggleCrop)
    };
    let mut h = harness();
    for (key, direction) in [
        (Key::ArrowLeft, -Vec2::X),
        (Key::ArrowRight, Vec2::X),
        (Key::ArrowUp, -Vec2::Y),
        (Key::ArrowDown, Vec2::Y),
    ] {
        h.key(key, Modifiers::NONE, &mut |ui| {
            assert_eq!(take(ui.ctx(), &arrows), Some(direction));
        });
    }
}

#[test]
fn rings_show_only_after_keys() {
    let mut h = harness();
    let mut run = |ui: &mut Ui| focus::begin_frame(ui.ctx());
    h.settle(1, &mut run);
    assert!(!focus::visible(&h.ctx));
    h.key(Key::Tab, Modifiers::NONE, &mut run);
    assert!(focus::visible(&h.ctx));
    h.click(pos2(10.0, 10.0), &mut run);
    assert!(!focus::visible(&h.ctx));
}

#[test]
fn up_and_down_move_between_list_items() {
    use super::list_nav::ListNav;
    let mut h = harness();
    let mut run = |ui: &mut Ui| {
        let mut nav = ListNav::new(ui.ctx());
        for i in 0..3 {
            let rect = Rect::from_min_size(pos2(0.0, 30.0 * i as f32), vec2(80.0, 20.0));
            nav.item(&ui.put(rect, egui::Button::new("item")));
        }
        nav.finish(ui.ctx());
        focus::end_frame(ui.ctx());
    };
    let mut top_after = |key| {
        h.press(key, Modifiers::NONE, &mut run)
            .map(|f| f.rect.top())
    };
    let tops = [
        Key::Tab,
        Key::ArrowDown,
        Key::ArrowDown,
        Key::ArrowDown,
        Key::ArrowUp,
        Key::Home,
        Key::End,
    ];
    let tops = tops.map(&mut top_after);
    // Down stays on the last item; Home and End go to the ends.
    assert_eq!(tops, [0.0, 30.0, 60.0, 60.0, 30.0, 0.0, 60.0].map(Some));
}

#[test]
fn every_shortcut_is_bound_once() {
    for (i, a) in TABLE.iter().enumerate() {
        let clash = TABLE[i + 1..]
            .iter()
            .find(|b| (b.keys, b.mods, b.scope) == (a.keys, a.mods, a.scope));
        assert!(
            clash.is_none(),
            "{:?} and {:?} share their keys",
            a.action,
            clash.map(|b| b.action)
        );
    }
}
