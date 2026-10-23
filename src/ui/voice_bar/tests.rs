use super::*;

#[test]
fn closed_until_toggled_and_closing_releases_the_microphone() {
    let mut voice = VoiceBar::default();
    assert!(!voice.is_open() && !voice.is_listening());
    // Opens even without a microphone (CI): the failure is a toast.
    voice.toggle();
    assert!(voice.is_open());
    voice.toggle();
    assert!(!voice.is_open() && !voice.is_listening());
}

/// A provider that replays `events`.
struct Replay(std::collections::VecDeque<crate::voice::stt::SttEvent>);

impl SpeechToText for Replay {
    fn push(&mut self, _: &[f32]) {}
    fn finish(&mut self) {}
    fn poll(&mut self) -> Option<crate::voice::stt::SttEvent> {
        self.0.pop_front()
    }
}

#[test]
fn a_provider_that_fails_to_get_ready_is_dropped() {
    use crate::voice::stt::SttEvent::{Failed, Preparing, Ready};
    let ctx = egui::Context::default();
    // Failing while downloading: dropped, so listening again starts afresh.
    let mut voice = VoiceBar {
        stt: Some(Box::new(Replay(
            [Preparing(Some(0.4)), Failed("offline".into())].into(),
        ))),
        ..VoiceBar::default()
    };
    voice.update(&ctx);
    assert!(voice.stt.is_none());
    // Failing once ready (one transcription): kept, it still runs.
    voice.stt = Some(Box::new(Replay(
        [Preparing(None), Ready, Failed("once".into())].into(),
    )));
    voice.update(&ctx);
    assert!(voice.stt.is_some());
}
