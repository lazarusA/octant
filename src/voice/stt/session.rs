//! A [`WindowModel`] given audio and stops one input at a time, the
//! utterances cut by the [`Segmenter`]: each utterance that ends is
//! transcribed for its final text at once; a partial text is only marked
//! due, and transcribed by [`Session::partial`] once the inputs waiting have
//! been taken, so a slow model skips stale partials instead of falling
//! behind; and only while the last transcription took at most
//! [`LIVE_BUDGET`], so a model too slow for live text (one browser CPU
//! thread) spends its time on final texts. Run by [`super::Windowed`] on the desktop and by the browser's
//! voice worker.

use super::segment::{Segmenter, Step};
use super::{SttEvent, WindowModel};
use web_time::{Duration, Instant};

/// The longest a transcription may take for partial texts to keep coming.
pub const LIVE_BUDGET: Duration = Duration::from_secs(1);

/// Samples of the audio heard while a model loads that are kept for it
/// (2 s): a quick load loses no first words, a long download transcribes
/// nothing stale.
pub const READY_TAIL: usize = super::SAMPLE_RATE as usize * 2;

pub enum Input {
    Audio(Vec<f32>),
    /// The speaker stopped.
    Finish,
}

pub struct Session<M> {
    model: M,
    seg: Segmenter,
    partial_due: bool,
    /// The last transcription was quick enough for partial texts.
    live: bool,
    budget: Duration,
}

impl<M: WindowModel> Session<M> {
    pub fn new(model: M) -> Self {
        Self {
            model,
            seg: Segmenter::default(),
            partial_due: false,
            live: true,
            budget: LIVE_BUDGET,
        }
    }

    /// A session that starts with final texts only, giving partial texts
    /// once a transcription shows the model is quick: for a model that is
    /// likely slow (the browser's), whose first partial would hold back the
    /// first final text.
    pub fn finals_first(model: M) -> Self {
        Self {
            live: false,
            ..Self::new(model)
        }
    }

    /// Takes `input`, giving `emit` the final text of every utterance it
    /// ends.
    pub fn take(&mut self, input: Input, emit: &mut impl FnMut(SttEvent)) {
        match input {
            Input::Audio(samples) => {
                let mut rest = samples.as_slice();
                while !rest.is_empty() {
                    let (step, used) = self.seg.feed(rest);
                    rest = &rest[used..];
                    match step {
                        Step::Final => self.finish_utterance(emit),
                        Step::Partial => self.partial_due = true,
                        Step::Wait => {}
                    }
                }
            }
            Input::Finish if self.seg.finish() == Step::Final => self.finish_utterance(emit),
            Input::Finish => {}
        }
    }

    /// Transcribes the utterance under way for a partial text, if one is due.
    pub fn partial(&mut self, emit: &mut impl FnMut(SttEvent)) {
        if std::mem::take(&mut self.partial_due) && self.live {
            let result = timed(
                &mut self.model,
                self.seg.audio(),
                self.budget,
                &mut self.live,
            );
            send(result, SttEvent::Partial, emit);
        }
    }

    fn finish_utterance(&mut self, emit: &mut impl FnMut(SttEvent)) {
        self.partial_due = false;
        let audio = self.seg.take();
        let result = timed(&mut self.model, &audio, self.budget, &mut self.live);
        send(result, SttEvent::Final, emit);
    }
}

/// `model`'s text for `audio`, setting `live` to whether it came within
/// `budget`.
fn timed(
    model: &mut impl WindowModel,
    audio: &[f32],
    budget: Duration,
    live: &mut bool,
) -> Result<String, String> {
    let started = Instant::now();
    let result = model.transcribe(audio);
    *live = started.elapsed() <= budget;
    result
}

/// `result` as `kind`, a failure as `Failed`; empty text is not sent.
fn send(
    result: Result<String, String>,
    kind: fn(String) -> SttEvent,
    emit: &mut impl FnMut(SttEvent),
) {
    match result {
        Ok(text) if text.trim().is_empty() => {}
        Ok(text) => emit(kind(text.trim().to_string())),
        Err(e) => emit(SttEvent::Failed(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voice::stt::SAMPLE_RATE;

    /// Says "slow" or "quick" after taking that long.
    struct Model {
        delay: Duration,
    }

    impl WindowModel for Model {
        fn transcribe(&mut self, _: &[f32]) -> Result<String, String> {
            std::thread::sleep(self.delay);
            Ok(if self.delay > Duration::ZERO {
                "slow"
            } else {
                "quick"
            }
            .into())
        }
    }

    /// The events of half a second of speech, then a partial.
    fn half_second(session: &mut Session<Model>) -> Vec<SttEvent> {
        let mut events = Vec::new();
        let mut emit = |e| events.push(e);
        session.take(Input::Audio(vec![0.3; SAMPLE_RATE as usize / 2]), &mut emit);
        session.partial(&mut emit);
        events
    }

    #[test]
    fn a_slow_model_gives_final_texts_only() {
        let mut session = Session::new(Model {
            delay: Duration::from_millis(30),
        });
        session.budget = Duration::from_millis(10);
        assert_eq!(
            half_second(&mut session),
            [SttEvent::Partial("slow".into())]
        );
        assert!(
            half_second(&mut session).is_empty(),
            "too slow for partials"
        );
        let mut events = Vec::new();
        session.take(Input::Finish, &mut |e| events.push(e));
        assert_eq!(events, [SttEvent::Final("slow".into())]);
    }

    #[test]
    fn finals_first_waits_for_a_quick_transcription() {
        let mut session = Session::finals_first(Model {
            delay: Duration::ZERO,
        });
        assert!(half_second(&mut session).is_empty());
        session.take(Input::Finish, &mut |_| {});
        assert_eq!(
            half_second(&mut session),
            [SttEvent::Partial("quick".into())]
        );
    }

    #[test]
    fn a_quick_transcription_brings_partials_back() {
        let mut session = Session::new(Model {
            delay: Duration::from_millis(30),
        });
        session.budget = Duration::from_millis(10);
        half_second(&mut session);
        session.model.delay = Duration::ZERO;
        session.take(Input::Finish, &mut |_| {});
        assert_eq!(
            half_second(&mut session),
            [SttEvent::Partial("quick".into())]
        );
    }
}
