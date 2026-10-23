//! Speech to text behind one interface, whatever transcribes: a local model
//! or a service. The voice bar only sees [`SpeechToText`]; a provider owns
//! its threads or connection. Models that transcribe a stretch of audio at
//! a time implement [`WindowModel`] and run inside [`Windowed`], which cuts
//! the audio into utterances and asks for live partial text.

// Threads: desktop only until the web runs models in a worker.
#[cfg(not(target_arch = "wasm32"))]
mod segment;
#[cfg(not(target_arch = "wasm32"))]
mod window;

#[cfg(not(target_arch = "wasm32"))]
pub use window::{WindowModel, Windowed};

/// Samples per second every provider is given (mono).
pub const SAMPLE_RATE: u32 = 16_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SttEvent {
    /// The words so far of the utterance being spoken; replaces the last.
    Partial(String),
    /// An utterance's final text.
    Final(String),
    /// Transcription failed; the message says why.
    Failed(String),
}

pub trait SpeechToText {
    /// Adds [`SAMPLE_RATE`] mono samples.
    fn push(&mut self, samples: &[f32]);
    /// The speaker stopped: the utterance under way ends with a final text.
    fn finish(&mut self);
    /// The next event, without waiting.
    fn poll(&mut self) -> Option<SttEvent>;
}

/// The provider the user chose, ready to listen; `None` while none is set
/// up (Whisper on candle comes next, `VOICE_PLAN.md`).
pub fn open() -> Option<Box<dyn SpeechToText>> {
    None
}
