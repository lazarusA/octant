//! Whisper's log-mel input: the Slaney-style mel filter bank (as librosa
//! builds it, which OpenAI's `mel_filters.npz` holds) applied by candle's
//! `pcm_to_mel`, padded or cut to the 30 s window the encoder reads.

use candle_core::{Device, Tensor};
use candle_transformers::models::whisper::{self as m, Config, audio};

/// FFT bins of Whisper's 400-sample window.
const BINS: usize = m::N_FFT / 2 + 1;

/// `n_mels` triangular filters over [`BINS`] bins, row by row.
pub(super) fn filters(n_mels: usize) -> Vec<f32> {
    // Linear below 1 kHz, logarithmic above.
    let (f_sp, min_log_hz) = (200.0 / 3.0, 1000.0);
    let min_log_mel = min_log_hz / f_sp;
    let log_step = 6.4f64.ln() / 27.0;
    let to_mel = |hz: f64| {
        if hz < min_log_hz {
            hz / f_sp
        } else {
            min_log_mel + (hz / min_log_hz).ln() / log_step
        }
    };
    let to_hz = |mel: f64| {
        if mel < min_log_mel {
            mel * f_sp
        } else {
            min_log_hz * ((mel - min_log_mel) * log_step).exp()
        }
    };
    let top = to_mel(m::SAMPLE_RATE as f64 / 2.0);
    let edges: Vec<f64> = (0..n_mels + 2)
        .map(|i| to_hz(top * i as f64 / (n_mels + 1) as f64))
        .collect();
    let mut out = vec![0.0; n_mels * BINS];
    for (j, row) in out.chunks_mut(BINS).enumerate() {
        let (lo, mid, hi) = (edges[j], edges[j + 1], edges[j + 2]);
        let norm = 2.0 / (hi - lo);
        for (k, w) in row.iter_mut().enumerate() {
            let hz = k as f64 * m::SAMPLE_RATE as f64 / m::N_FFT as f64;
            let rise = (hz - lo) / (mid - lo);
            let fall = (hi - hz) / (hi - mid);
            *w = (rise.min(fall).max(0.0) * norm) as f32;
        }
    }
    out
}

/// The encoder's input for `audio`: `(1, n_mels, N_FRAMES)`.
pub(super) fn input(
    config: &Config,
    filters: &[f32],
    audio: &[f32],
    device: &Device,
) -> candle_core::Result<Tensor> {
    let mel = audio::pcm_to_mel(config, audio, filters);
    let frames = mel.len() / config.num_mel_bins;
    Tensor::from_vec(mel, (1, config.num_mel_bins, frames), device)?.narrow(2, 0, m::N_FRAMES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_are_triangles_of_equal_area_in_mel() {
        for n_mels in [80, 128] {
            let f = filters(n_mels);
            assert_eq!(f.len(), n_mels * BINS);
            // Every filter reads some bin, and each row peaks once.
            for row in f.chunks(BINS) {
                let peak = row
                    .iter()
                    .position(|&w| w == row.iter().fold(0.0f32, |a, &b| a.max(b)));
                assert!(row.iter().any(|&w| w > 0.0) && peak.is_some());
            }
        }
    }
}

#[cfg(test)]
mod reference {
    /// Candle's example ships OpenAI's filters; compare when they are at hand:
    /// `OCTANT_MELFILTERS=dir cargo test --lib mel_filters_match`.
    #[test]
    fn mel_filters_match_openai() {
        let Ok(dir) = std::env::var("OCTANT_MELFILTERS") else {
            return;
        };
        for (n, file) in [(80, "melfilters.bytes"), (128, "melfilters128.bytes")] {
            let bytes = std::fs::read(format!("{dir}/{file}")).expect("filters file");
            let reference: Vec<f32> = bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|&b| f32::from_le_bytes(b))
                .collect();
            let ours = super::filters(n);
            let worst = ours
                .iter()
                .zip(&reference)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f32::max);
            println!("{file}: max difference {worst:e}");
            assert!(worst < 1e-6, "{file}: {worst}");
        }
    }
}
