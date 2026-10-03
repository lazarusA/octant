//! Keyboard navigation of the variable tree, driven through the headless
//! harness in `tests.rs`. Fixture rows with every folder open:
//! `/` > lat(0), ocean > sst(1), ocean/deep > temp(2), land > lai(3).

use super::nav::{NodeKey, folder_id, row_id};
use super::tests::{click, focus, focused, frame, is_open, press};
use egui::Key;

fn row(key: NodeKey<'_>) -> Option<egui::Id> {
    Some(row_id(key, false))
}

#[test]
fn up_down_follow_visible_rows_and_skip_closed_folders() {
    let ctx = egui::Context::default();
    frame(&ctx, Vec::new(), false);
    focus(&ctx, NodeKey::Folder(""), false);

    press(&ctx, Key::ArrowDown, false);
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Folder("ocean")),
        "closed `/` hides lat"
    );
    press(&ctx, Key::ArrowDown, false);
    assert_eq!(focused(&ctx), row(NodeKey::Folder("land")));
    press(&ctx, Key::ArrowDown, false);
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Folder("land")),
        "last row stays put"
    );
    press(&ctx, Key::ArrowUp, false);
    assert_eq!(focused(&ctx), row(NodeKey::Folder("ocean")));
    press(&ctx, Key::End, false);
    assert_eq!(focused(&ctx), row(NodeKey::Folder("land")));
    press(&ctx, Key::Home, false);
    assert_eq!(focused(&ctx), row(NodeKey::Folder("")));
}

#[test]
fn right_opens_a_folder_then_enters_it() {
    let ctx = egui::Context::default();
    frame(&ctx, Vec::new(), false);
    focus(&ctx, NodeKey::Folder("ocean"), false);

    let f = press(&ctx, Key::ArrowRight, false);
    assert!(is_open(&ctx, f.ocean_id), "first Right opens");
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Folder("ocean")),
        "and keeps focus"
    );

    press(&ctx, Key::ArrowRight, false);
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Variable(1)),
        "second Right enters"
    );

    press(&ctx, Key::ArrowDown, false);
    assert_eq!(focused(&ctx), row(NodeKey::Folder("ocean/deep")));
    press(&ctx, Key::ArrowRight, false);
    press(&ctx, Key::ArrowRight, false);
    assert_eq!(focused(&ctx), row(NodeKey::Variable(2)));

    press(&ctx, Key::ArrowRight, false);
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Variable(2)),
        "Right on a variable is a no-op"
    );
}

#[test]
fn left_goes_to_parent_then_closes_it() {
    let ctx = egui::Context::default();
    frame(&ctx, Vec::new(), false);
    focus(&ctx, NodeKey::Folder("ocean"), false);
    for _ in 0..2 {
        press(&ctx, Key::ArrowRight, false);
    }
    press(&ctx, Key::ArrowDown, false);
    for _ in 0..2 {
        press(&ctx, Key::ArrowRight, false);
    }
    assert_eq!(focused(&ctx), row(NodeKey::Variable(2)));

    press(&ctx, Key::ArrowLeft, false);
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Folder("ocean/deep")),
        "variable to parent"
    );
    press(&ctx, Key::ArrowLeft, false);
    assert!(
        !is_open(&ctx, folder_id("ocean/deep", false)),
        "open folder closes"
    );
    press(&ctx, Key::ArrowLeft, false);
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Folder("ocean")),
        "closed folder to parent"
    );
    let f = press(&ctx, Key::ArrowLeft, false);
    assert!(!is_open(&ctx, f.ocean_id));
    press(&ctx, Key::ArrowLeft, false);
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Folder("ocean")),
        "top level stays put"
    );
}

#[test]
fn enter_selects_the_focused_variable() {
    let ctx = egui::Context::default();
    frame(&ctx, Vec::new(), true);
    focus(&ctx, NodeKey::Variable(1), true);
    frame(&ctx, Vec::new(), true);

    let f = press(&ctx, Key::Enter, true);
    assert_eq!(f.selected, Some(1));
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
fn keys_continue_from_a_mouse_click() {
    let ctx = egui::Context::default();
    let f = frame(&ctx, Vec::new(), false);
    let at_row = |i: f32| egui::pos2(f.origin.x + 80.0, f.origin.y + f.row_step * (i + 0.5));

    // Click `ocean` (row 1) open, then click `sst` (row 2) with the mouse.
    click(&ctx, at_row(1.0), false);
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Folder("ocean")),
        "click focuses the folder"
    );
    let f = click(&ctx, at_row(2.0), false);
    assert_eq!(f.selected, Some(1));
    assert_eq!(
        focused(&ctx),
        row(NodeKey::Variable(1)),
        "click focuses the variable"
    );

    press(&ctx, Key::ArrowDown, false);
    assert_eq!(focused(&ctx), row(NodeKey::Folder("ocean/deep")));
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
