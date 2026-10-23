//! Voice input: the microphone, its spectrum, and speech to text behind a
//! provider-independent interface (`stt`). Providers and the action layer
//! come next (`VOICE_PLAN.md`).

mod capture;
pub mod resample;
pub mod spectrum;
pub mod stt;
pub mod whisper;

pub use capture::Capture;

/// A JavaScript error's message, for reports.
#[cfg(target_arch = "wasm32")]
pub fn js_text(value: wasm_bindgen::JsValue) -> String {
    use wasm_bindgen::JsCast;
    value
        .dyn_ref::<js_sys::Error>()
        .map(|e| String::from(e.message()))
        .or_else(|| value.as_string())
        .unwrap_or_else(|| format!("{value:?}"))
}
