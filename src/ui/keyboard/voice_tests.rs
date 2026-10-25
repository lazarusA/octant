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

#[test]
fn commands_waiting_when_the_bar_is_closed_never_run() {
    let (mut app, mut h) = window(false);
    // The time step waits for sst's coordinates, which are being read.
    let first = app.run_utterance("select sst, then time 2");
    assert_eq!(first.len(), 1, "{first:?}");
    h.settle(2, &mut run(&mut app));
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while app.coordinate_loader.pending_count() > 0 {
        assert!(
            web_time::Instant::now() < deadline,
            "coordinates never came"
        );
        app.poll_coordinate_results();
        std::thread::yield_now();
    }
    assert_eq!(
        app.resume_commands(),
        vec![],
        "dropped when the closed bar was drawn"
    );
}
