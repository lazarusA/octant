//! The keymap's matching, focus visibility, the focus kept across frames,
//! and region jumps.

use super::focus;
use super::keymap::{Action, Keys, Shortcut, TABLE, take};
use super::regions::{self, Region};
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

/// Three buttons standing for the regions, top to bottom; F6 handled as the
/// dispatcher does, before anything draws.
fn regions_frame(ui: &mut Ui, jump: Option<bool>) {
    regions::begin_frame(ui.ctx());
    if let Some(forward) = jump {
        regions::cycle(ui.ctx(), forward);
    }
    for (i, region) in [Region::TopBar, Region::Canvas, Region::BottomBar]
        .into_iter()
        .enumerate()
    {
        let rect = Rect::from_min_size(pos2(0.0, 40.0 * i as f32), vec2(100.0, 30.0));
        let button = ui.put(rect, egui::Button::new("x"));
        regions::register(ui.ctx(), region, button.id, button.layer_id, rect);
    }
    focus::end_frame(ui.ctx());
}

#[test]
fn f6_cycles_regions_in_visual_order() {
    let mut h = harness();
    h.settle(2, &mut |ui| regions_frame(ui, None));
    let mut visited = Vec::new();
    for _ in 0..4 {
        h.frame(Vec::new(), &mut |ui| regions_frame(ui, Some(true)));
        h.settle(1, &mut |ui| regions_frame(ui, None));
        let last = focus::last(&h.ctx).expect("a region is focused");
        visited.push(last.rect.top());
    }
    assert_eq!(visited, [0.0, 40.0, 80.0, 0.0], "wraps after the last");

    h.frame(Vec::new(), &mut |ui| regions_frame(ui, Some(false)));
    h.settle(1, &mut |ui| regions_frame(ui, None));
    let last = focus::last(&h.ctx).expect("a region is focused");
    assert_eq!(last.rect.top(), 80.0, "Shift+F6 goes back");
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
        h.key(key, Modifiers::NONE, &mut run);
        h.settle(1, &mut run);
        focus::last(&h.ctx).expect("an item is focused").rect.top()
    };
    assert_eq!(top_after(Key::Tab), 0.0);
    assert_eq!(top_after(Key::ArrowDown), 30.0);
    assert_eq!(top_after(Key::ArrowDown), 60.0);
    assert_eq!(top_after(Key::ArrowDown), 60.0, "stays on the last item");
    assert_eq!(top_after(Key::ArrowUp), 30.0);
}
