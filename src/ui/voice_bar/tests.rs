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

#[test]
fn listening_again_keeps_the_loading_state_and_dropping_forgets_it() {
    use crate::voice::stt::SttEvent::{Failed, Preparing};
    let ctx = egui::Context::default();
    let mut voice = VoiceBar {
        stt: Some(Box::new(Replay([Preparing(None)].into()))),
        ..VoiceBar::default()
    };
    voice.update(&ctx);
    // Listening again clears the text only: still loading, and a failure
    // while loading still drops the provider.
    voice.transcript.clear();
    assert!(voice.transcript.is_preparing());
    voice.stt = Some(Box::new(Replay([Failed("no memory".into())].into())));
    voice.update(&ctx);
    assert!(voice.stt.is_none());
    // A provider dropped while it got ready (the microphone refused) leaves
    // no download showing.
    voice.stt = Some(Box::new(Replay([Preparing(Some(0.03))].into())));
    voice.update(&ctx);
    assert!(voice.transcript.is_preparing());
    voice.release();
    assert!(!voice.transcript.is_preparing());
}
