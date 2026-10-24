//! [`Windowed`]: a [`WindowModel`] in a [`Session`] on its own thread. The
//! model is made on that thread first (a download, a
//! load), reporting through [`Prep`]; what is heard meanwhile is kept here
//! as a [`Tail`] (its last seconds and stops), so a quick load loses no
//! first words and a long download neither piles up audio nor transcribes
//! anything stale. Audio that arrives while the model works is taken before
//! a partial text is made.

use super::session::{Input as Msg, Session};
use super::tail::Tail;
use super::{SpeechToText, SttEvent, WindowModel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};

/// What making a model may report, and whether to give up.
pub struct Prep {
    events: Sender<SttEvent>,
    cancel: Arc<AtomicBool>,
}

impl Prep {
    pub fn report(&self, event: SttEvent) {
        let _ = self.events.send(event);
    }

    /// The provider was dropped: stop making the model.
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

/// Dropping it cancels the model's making, or stops the thread once the
/// model returns.
pub struct Windowed {
    to_worker: Sender<Msg>,
    events: Receiver<SttEvent>,
    cancel: Arc<AtomicBool>,
    /// What was heard before the model said it is ready; `None` after.
    tail: Option<Tail>,
}

impl Windowed {
    /// Makes the model with `make` on a new thread, then transcribes; a
    /// failure to make it is reported unless cancelled.
    pub fn spawn<M: WindowModel>(
        make: impl FnOnce(&Prep) -> Result<M, String> + Send + 'static,
    ) -> Self {
        let (to_worker, inbox) = mpsc::channel();
        let (outbox, events) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let prep = Prep {
            events: outbox.clone(),
            cancel: Arc::clone(&cancel),
        };
        std::thread::spawn(move || match make(&prep) {
            Ok(model) => {
                prep.report(SttEvent::Ready);
                run(model, &inbox, &outbox);
            }
            Err(e) if !prep.cancelled() => prep.report(SttEvent::Failed(e)),
            Err(_) => {}
        });
        Self {
            to_worker,
            events,
            cancel,
            tail: Some(Tail::default()),
        }
    }

    fn send(&mut self, msg: Msg) {
        match &mut self.tail {
            Some(tail) => tail.push(msg),
            None => {
                let _ = self.to_worker.send(msg);
            }
        }
    }
}

impl Drop for Windowed {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

impl SpeechToText for Windowed {
    fn push(&mut self, samples: &[f32]) {
        if !samples.is_empty() {
            self.send(Msg::Audio(samples.to_vec()));
        }
    }

    fn finish(&mut self) {
        self.send(Msg::Finish);
    }

    fn set_prompt(&mut self, prompt: &str) {
        self.send(Msg::Prompt(prompt.to_string()));
    }

    fn poll(&mut self) -> Option<SttEvent> {
        let event = self.events.try_recv().ok();
        if event == Some(SttEvent::Ready)
            && let Some(mut tail) = self.tail.take()
        {
            for msg in tail.take() {
                let _ = self.to_worker.send(msg);
            }
        }
        event
    }
}

/// Transcribes what arrives in `inbox` until the provider is dropped.
fn run(model: impl WindowModel, inbox: &Receiver<Msg>, outbox: &Sender<SttEvent>) {
    let mut session = Session::new(model);
    let mut emit = |event| {
        let _ = outbox.send(event);
    };
    while let Ok(first) = inbox.recv() {
        for msg in std::iter::once(first).chain(inbox.try_iter()) {
            session.take(msg, &mut emit);
        }
        session.partial(&mut emit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voice::stt::SAMPLE_RATE;
    use std::time::{Duration, Instant};

    /// Names the seconds of audio it was given.
    struct Seconds;

    impl WindowModel for Seconds {
        fn transcribe(&mut self, audio: &[f32]) -> Result<String, String> {
            Ok(format!("{:.1}", audio.len() as f32 / SAMPLE_RATE as f32))
        }
    }

    /// A provider of [`Seconds`] once it says it is ready.
    fn ready() -> Windowed {
        let mut stt = Windowed::spawn(|_| Ok(Seconds));
        let deadline = Instant::now() + Duration::from_secs(5);
        while stt.poll() != Some(SttEvent::Ready) {
            assert!(Instant::now() < deadline, "never ready");
            std::thread::sleep(Duration::from_millis(1));
        }
        stt
    }

    #[test]
    fn audio_from_before_ready_keeps_its_last_seconds() {
        let mut stt = Windowed::spawn(|_| {
            std::thread::sleep(Duration::from_millis(200));
            Ok(Seconds)
        });
        stt.push(&vec![0.3; SAMPLE_RATE as usize * 5]);
        // Stopped before ready: the stop still ends the utterance.
        stt.finish();
        let events = events_until_final(&mut stt);
        assert_eq!(events.last(), Some(&SttEvent::Final("2.0".into())));
    }

    fn events_until_final(stt: &mut Windowed) -> Vec<SttEvent> {
        let mut events = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match stt.poll() {
                Some(e @ SttEvent::Final(_)) => {
                    events.push(e);
                    return events;
                }
                Some(e) => events.push(e),
                None => std::thread::sleep(Duration::from_millis(5)),
            }
        }
        panic!("no final text: {events:?}");
    }

    #[test]
    fn speech_then_a_pause_comes_back_as_partials_and_a_final() {
        let mut stt = ready();
        let second = SAMPLE_RATE as usize;
        stt.push(&vec![0.3; second * 2]);
        stt.push(&vec![0.0; second]);
        let events = events_until_final(&mut stt);
        // The pause ends the utterance after 0.8 s of it.
        assert_eq!(events.last(), Some(&SttEvent::Final("2.8".into())));
    }

    #[test]
    fn finishing_ends_the_utterance() {
        let mut stt = ready();
        stt.push(&vec![0.3; SAMPLE_RATE as usize]);
        stt.finish();
        let events = events_until_final(&mut stt);
        assert_eq!(events.last(), Some(&SttEvent::Final("1.0".into())));
    }
}
