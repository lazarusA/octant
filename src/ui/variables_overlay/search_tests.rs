//! Focus handoff between the search field and the variable tree: Down and
//! Enter from the field, Up and Escape back to it, and modifier combinations
//! that must stay in the field.

use super::nav::{NodeKey, row_id};
use super::tests::{focused, frame, press, press_with};
use egui::Key;

fn row(key: NodeKey<'_>) -> Option<egui::Id> {
    Some(row_id(key, false))
}

#[test]
fn search_field_and_tree_hand_focus_back_and_forth() {
    let ctx = egui::Context::default();
    let f = frame(&ctx, Vec::new(), false);
    ctx.memory_mut(|m| m.request_focus(f.search_id));
    frame(&ctx, Vec::new(), false);

    press(&ctx, Key::ArrowDown, false);
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Folder("")),
        "Down enters the tree"
    );
    press(&ctx, Key::ArrowUp, false);
    assert_eq!(
        focused(&ctx),
        Some(f.search_id),
        "Up on the first row returns"
    );

    press(&ctx, Key::ArrowDown, false);
    press(&ctx, Key::ArrowDown, false);
    press(&ctx, Key::Escape, false);
    assert_eq!(
        focused(&ctx),
        Some(f.search_id),
        "Escape returns from any row"
    );
}

#[test]
fn enter_in_search_focuses_the_first_matching_variable() {
    let ctx = egui::Context::default();
    let f = frame(&ctx, Vec::new(), true);
    ctx.memory_mut(|m| m.request_focus(f.search_id));
    frame(&ctx, Vec::new(), true);

    let f = press(&ctx, Key::Enter, true);
    assert_eq!(focused(&ctx), Some(row_id(NodeKey::Variable(0), true)));
    assert_eq!(
        f.selected, None,
        "several matches: focus only, no selection"
    );
}

#[test]
fn down_works_right_after_the_search_field_gets_focus() {
    let ctx = egui::Context::default();
    let f = frame(&ctx, Vec::new(), false);
    // What opening the overlay does: focus lands on the search field, no click.
    ctx.memory_mut(|m| m.request_focus(f.search_id));
    press(&ctx, Key::ArrowDown, false);
    assert_eq!(focused(&ctx), row(NodeKey::Folder("")));
}

#[test]
fn fast_up_or_escape_from_the_first_row_stops_at_search() {
    for key in [Key::ArrowUp, Key::Escape] {
        let ctx = egui::Context::default();
        let f = frame(&ctx, Vec::new(), false);
        ctx.memory_mut(|m| m.request_focus(f.search_id));
        frame(&ctx, Vec::new(), false);

        // No idle frame between presses, as with key repeat.
        press(&ctx, Key::ArrowDown, false);
        press(&ctx, key, false);
        assert_eq!(focused(&ctx), Some(f.search_id), "{key:?}");
    }
}

#[test]
fn modified_down_stays_in_the_search_field() {
    let ctx = egui::Context::default();
    let f = frame(&ctx, Vec::new(), false);
    ctx.memory_mut(|m| m.request_focus(f.search_id));
    frame(&ctx, Vec::new(), false);

    for modifiers in [egui::Modifiers::SHIFT, egui::Modifiers::COMMAND] {
        press_with(&ctx, Key::ArrowDown, modifiers, false);
        assert_eq!(focused(&ctx), Some(f.search_id), "{modifiers:?}");
    }
}
