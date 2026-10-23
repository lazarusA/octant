//! The default microphone as mono samples at its own rate. Recording lasts
//! as long as the [`Capture`]; dropping it releases the microphone.

#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::Capture;
#[cfg(target_arch = "wasm32")]
pub use web::Capture;

/// Samples kept for a reader that stops draining (a hidden window): about
/// two seconds at 48 kHz, the oldest dropped first.
const MAX_PENDING: usize = 96_000;

/// Appends `samples` to `pending`, keeping at most [`MAX_PENDING`].
fn keep_recent(pending: &mut Vec<f32>, samples: impl IntoIterator<Item = f32>) {
    pending.extend(samples);
    if pending.len() > MAX_PENDING {
        let excess = pending.len() - MAX_PENDING;
        pending.drain(..excess);
    }
}

/// `cargo test --lib probe_microphone -- --ignored --nocapture`: a peak of
/// 0 means the system gives silence (no microphone permission).
#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "records the default microphone"]
    fn probe_microphone() {
        let capture = super::Capture::start().expect("microphone");
        std::thread::sleep(std::time::Duration::from_secs(2));
        let mut samples = Vec::new();
        capture.drain(&mut samples);
        let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        println!(
            "rate {} samples {} peak {peak}",
            capture.sample_rate(),
            samples.len()
        );
    }
}
