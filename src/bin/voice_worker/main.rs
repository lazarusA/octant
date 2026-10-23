//! The voice worker: Whisper in a browser Web Worker, so transcription
//! never blocks the page (`VOICE_PLAN.md` 4b). Trunk builds it as its own
//! `.wasm`, fetched only when the voice bar opens. The app sends it a `load`
//! command, audio and `finish` (`octant::voice::stt::wire`); it downloads the
//! model from OpenAI's repositories into the browser's cache (`files.rs`),
//! loads it quantized and answers with transcription events. Nothing on the
//! desktop.

#[cfg(target_arch = "wasm32")]
mod files;
#[cfg(target_arch = "wasm32")]
mod worker;

#[cfg(not(target_arch = "wasm32"))]
fn main() {}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    worker::start();
}
