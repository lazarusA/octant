//! Speech to text behind one interface, whatever transcribes: a local model
//! or a service. The voice bar only sees [`SpeechToText`]; a provider owns
//! its threads or connection. Models that transcribe a stretch of audio at
//! a time implement [`WindowModel`] and run inside [`Windowed`], which cuts
//! the audio into utterances and asks for live partial text.

mod segment;
mod session;
mod tail;
pub mod wire;
// Threads: the browser runs its model in the voice worker instead.
#[cfg(not(target_arch = "wasm32"))]
mod window;
#[cfg(target_arch = "wasm32")]
mod worker;

pub use session::{Input, Session};
pub use tail::Tail;

#[cfg(not(target_arch = "wasm32"))]
pub use window::{Prep, Windowed};

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

/// A model that transcribes a stretch of [`SAMPLE_RATE`] mono audio, run
/// by [`Windowed`] on the desktop and by the voice worker in the browser.
pub trait WindowModel {
    fn transcribe(&mut self, audio: &[f32]) -> Result<String, String>;
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

    /// The models offered here: in the browser only those small enough to
    /// download and run on one CPU thread.
    #[cfg(not(target_arch = "wasm32"))]
    pub const OFFERED: &[Self] = &Self::ALL;
    #[cfg(target_arch = "wasm32")]
    pub const OFFERED: &[Self] = &[Self::Tiny, Self::Base];

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

/// The browser runs Whisper in its voice worker.
#[cfg(target_arch = "wasm32")]
fn whisper(size: WhisperSize) -> Option<Box<dyn SpeechToText>> {
    match worker::WorkerStt::start(size) {
        Ok(stt) => Some(Box::new(stt)),
        Err(e) => {
            let title = crate::i18n::tr(crate::i18n::PlaybackKey::VoiceSttFailed);
            crate::ui::toast::report(crate::ui::toast::Severity::Error, title, e);
            None
        }
    }
}
