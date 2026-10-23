//! Voice input: the microphone and its spectrogram. Speech to text and the
//! action layer come later (`VOICE_PLAN.md`).

mod capture;
pub mod spectrum;

pub use capture::Capture;
