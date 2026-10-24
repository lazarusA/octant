//! M opens the voice bar to type a command; Enter runs it as a spoken one.

use super::test_support::{press, run, window};
use egui::{Event, Key, Modifiers};

#[test]
fn m_opens_the_command_field_and_enter_runs_what_is_typed() {
    let (mut app, mut h) = window(false);
    let focused = press(&mut h, &mut app, Key::M, Modifiers::NONE);
    assert!(
        app.voice.is_open() && !app.voice.is_listening(),
        "open, microphone off"
    );
    assert_eq!(
        focused.map(|f| f.id),
        Some(crate::ui::voice_bar::field_id())
    );
    h.frame(vec![Event::Text("select sst".into())], &mut run(&mut app));
    press(&mut h, &mut app, Key::Enter, Modifiers::NONE);
    let staged = app
        .selected_variable_info()
        .map(|v| v.name.trim_matches('/').to_string());
    assert_eq!(staged.as_deref(), Some("sst"));
    assert!(
        h.ctx.text_edit_focused(),
        "the field stays ready for the next"
    );
}
