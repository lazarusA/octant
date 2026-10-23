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
pub use window::{Prep, WindowModel, Windowed};

/// Samples per second every provider is given (mono).
pub const SAMPLE_RATE: u32 = 16_000;

#[derive(Clone, Debug, PartialEq)]
pub enum SttEvent {
    /// The words so far of the utterance being spoken; replaces the last.
    Partial(String),
    /// An utterance's final text.
    Final(String),
    /// Getting ready: downloading the model (the share done) or, `None`,
    /// loading it.
    Preparing(Option<f32>),
    /// Ready to transcribe.
    Ready,
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

/// Who transcribes. Other providers (services, the browser's) join here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    /// OpenAI's Whisper on this machine, through candle.
    Whisper(WhisperSize),
}

impl Default for Provider {
    fn default() -> Self {
        Self::Whisper(WhisperSize::default())
    }
}

/// Whisper's models, smallest first; larger ones are slower and more exact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WhisperSize {
    #[default]
    Tiny,
    Base,
    Small,
    Turbo,
}

impl WhisperSize {
    pub const ALL: [Self; 4] = [Self::Tiny, Self::Base, Self::Small, Self::Turbo];

    pub fn name(self) -> &'static str {
        match self {
            Self::Tiny => "tiny",
            Self::Base => "base",
            Self::Small => "small",
            Self::Turbo => "large-v3-turbo",
        }
    }
}

/// `provider`, ready to listen; `None` where it cannot run.
pub fn open(provider: Provider) -> Option<Box<dyn SpeechToText>> {
    match provider {
        Provider::Whisper(size) => whisper(size),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn whisper(size: WhisperSize) -> Option<Box<dyn SpeechToText>> {
    Some(Box::new(Windowed::spawn(move |prep: &Prep| {
        crate::voice::whisper::prepare(size, prep)
    })))
}

/// The browser has no threads to run Whisper on until it runs in a worker.
#[cfg(target_arch = "wasm32")]
fn whisper(_: WhisperSize) -> Option<Box<dyn SpeechToText>> {
    None
}
