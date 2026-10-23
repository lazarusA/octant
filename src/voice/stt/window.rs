//! [`Windowed`]: a [`WindowModel`] on its own thread, given utterances cut by
//! the [`Segmenter`]. The model is made on that thread first (a download, a
//! load), reporting through [`Prep`]; audio heard meanwhile is dropped. Audio
//! that arrives while the model works is gathered first, so a slow model
//! skips stale partials instead of falling behind.

use super::segment::{Segmenter, Step};
use super::{SpeechToText, SttEvent};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};

/// A model that transcribes a stretch of [`super::SAMPLE_RATE`] mono audio.
pub trait WindowModel: Send + 'static {
    fn transcribe(&mut self, audio: &[f32]) -> Result<String, String>;
}

enum Msg {
    Audio(Vec<f32>),
    Finish,
}

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
                inbox.try_iter().for_each(drop);
                run(model, &inbox, &outbox);
            }
            Err(e) if !prep.cancelled() => prep.report(SttEvent::Failed(e)),
            Err(_) => {}
        });
        Self {
            to_worker,
            events,
            cancel,
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
            let _ = self.to_worker.send(Msg::Audio(samples.to_vec()));
        }
    }

    fn finish(&mut self) {
        let _ = self.to_worker.send(Msg::Finish);
    }

    fn poll(&mut self) -> Option<SttEvent> {
        self.events.try_recv().ok()
    }
}

fn run(mut model: impl WindowModel, inbox: &Receiver<Msg>, outbox: &Sender<SttEvent>) {
    let mut seg = Segmenter::default();
    let send = |result: Result<String, String>, event: fn(String) -> SttEvent| {
        let event = match result {
            Ok(text) if text.trim().is_empty() => return,
            Ok(text) => event(text.trim().to_string()),
            Err(e) => SttEvent::Failed(e),
        };
        let _ = outbox.send(event);
    };
    while let Ok(first) = inbox.recv() {
        let mut partial = false;
        for msg in std::iter::once(first).chain(inbox.try_iter()) {
            let finals = match msg {
                Msg::Audio(samples) => {
                    let mut rest = samples.as_slice();
                    let mut finals = Vec::new();
                    while !rest.is_empty() {
                        let (step, used) = seg.feed(rest);
                        rest = &rest[used..];
                        match step {
                            Step::Final => finals.push(seg.take()),
                            Step::Partial => partial = true,
                            Step::Wait => {}
                        }
                    }
                    finals
                }
                Msg::Finish if seg.finish() == Step::Final => vec![seg.take()],
                Msg::Finish => Vec::new(),
            };
            for audio in finals {
                partial = false;
                send(model.transcribe(&audio), SttEvent::Final);
            }
        }
        if partial {
            send(model.transcribe(seg.audio()), SttEvent::Partial);
        }
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
        let mut stt = Windowed::spawn(|_| Ok(Seconds));
        let second = SAMPLE_RATE as usize;
        stt.push(&vec![0.3; second * 2]);
        stt.push(&vec![0.0; second]);
        let events = events_until_final(&mut stt);
        // The pause ends the utterance after 0.8 s of it.
        assert_eq!(events.last(), Some(&SttEvent::Final("2.8".into())));
    }

    #[test]
    fn finishing_ends_the_utterance() {
        let mut stt = Windowed::spawn(|_| Ok(Seconds));
        stt.push(&vec![0.3; SAMPLE_RATE as usize]);
        stt.finish();
        let events = events_until_final(&mut stt);
        assert_eq!(events.last(), Some(&SttEvent::Final("1.0".into())));
    }
}
