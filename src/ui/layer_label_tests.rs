//! The layer label editor keeps no text while idle, and commits an edit
//! when the field loses focus.

use super::layer_label::LabelEditor;
use crate::app::layers::{Layer, LayerStack};
use egui::{Event, Key, PointerButton, Pos2, RawInput, Rect, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 100.0));

fn editor() -> LabelEditor {
    LabelEditor {
        id: egui::Id::new("test_label"),
        width: 200.0,
        framed: true,
    }
}

/// Runs one frame with `events`; returns the editor's result and the center
/// of its widget.
fn frame(ctx: &egui::Context, layer: &Layer, events: Vec<Event>) -> (Option<Option<String>>, Pos2) {
    let input = RawInput {
        screen_rect: Some(SCREEN),
        events,
        ..Default::default()
    };
    let mut out = (None, Pos2::ZERO);
    let mut output = ctx.run_ui(input, |ui| {
        let top = ui.cursor().min;
        out.0 = editor().show(ui, layer);
        out.1 = top + egui::vec2(100.0, 10.0);
    });
    output.textures_delta.clear();
    out
}

fn press(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    }
}

#[test]
fn idle_frames_keep_no_edit_text() {
    let stack = LayerStack::default();
    let ctx = egui::Context::default();
    for _ in 0..3 {
        assert_eq!(frame(&ctx, &stack.base, Vec::new()).0, None);
    }
    let editing = ctx.data(|d| d.get_temp::<String>(editor().id));
    assert_eq!(editing, None);
}

#[test]
fn an_edit_commits_when_enter_ends_it() {
    let stack = LayerStack::default();
    let ctx = egui::Context::default();
    let (_, at) = frame(&ctx, &stack.base, Vec::new());
    frame(
        &ctx,
        &stack.base,
        vec![Event::PointerMoved(at), press(at, true)],
    );
    frame(&ctx, &stack.base, vec![press(at, false)]);
    frame(&ctx, &stack.base, Vec::new());
    frame(&ctx, &stack.base, vec![Event::Text(" (K)".to_string())]);
    let enter = Event::Key {
        key: Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    };
    let (edited, _) = frame(&ctx, &stack.base, vec![enter]);
    assert_eq!(edited, Some(Some("Scalar Field (K)".to_string())));
    let editing = ctx.data(|d| d.get_temp::<String>(editor().id));
    assert_eq!(editing, None, "the edit text is dropped");
}
