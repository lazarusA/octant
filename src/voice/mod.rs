//! Voice input: the microphone, its spectrum, and speech to text behind a
//! provider-independent interface (`stt`). Providers and the action layer
//! come next (`VOICE_PLAN.md`).

mod capture;
pub mod resample;
pub mod spectrum;
pub mod stt;
#[cfg(not(target_arch = "wasm32"))]
pub mod whisper;

pub use capture::Capture;
