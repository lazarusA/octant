//! Whisper's log-mel input, as OpenAI computes it: 400-sample Hann windows
//! every 160 samples, the power of each frequency bin, the Slaney-style mel
//! filter bank (as librosa builds it, which OpenAI's `mel_filters.npz`
//! holds), log10, then Whisper's clamp to 8 below the maximum. Done as candle
//! matrix products on the model's device, not with candle's `pcm_to_mel`,
//! which always starts threads (the browser has none) and doubles the power
//! of the middle bins as whisper.cpp does. Only the frames that hold audio
//! are transformed; the rest of the 30 s window is silence.

use candle_core::{Device, Tensor};
use candle_transformers::models::whisper as m;

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

/// `log10` of the floor every power is clamped to, which silence reads.
const SILENCE: f32 = -10.0;

/// The tensors the transform reuses, on the model's device.
pub(super) struct Mel {
    n_mels: usize,
    /// Real and imaginary DFT bases, `(N_FFT, BINS)`.
    cos: Tensor,
    sin: Tensor,
    /// The filter bank transposed, `(BINS, n_mels)`.
    filters: Tensor,
    hann: Vec<f32>,
}

impl Mel {
    pub(super) fn new(n_mels: usize, device: &Device) -> candle_core::Result<Self> {
        let angle = |n: usize, k: usize| {
            std::f32::consts::TAU * ((n * k) % m::N_FFT) as f32 / m::N_FFT as f32
        };
        let basis = |f: fn(f32) -> f32| -> Vec<f32> {
            (0..m::N_FFT)
                .flat_map(|n| (0..BINS).map(move |k| f(angle(n, k))))
                .collect()
        };
        let shape = (m::N_FFT, BINS);
        Ok(Self {
            n_mels,
            cos: Tensor::from_vec(basis(f32::cos), shape, device)?,
            sin: Tensor::from_vec(basis(f32::sin), shape, device)?,
            filters: Tensor::from_vec(filters(n_mels), (n_mels, BINS), device)?
                .t()?
                .contiguous()?,
            hann: (0..m::N_FFT)
                .map(|n| 0.5 - 0.5 * (std::f32::consts::TAU * n as f32 / m::N_FFT as f32).cos())
                .collect(),
        })
    }

    /// The encoder's input for `audio`, cut to 30 s: `(1, n_mels, N_FRAMES)`.
    /// Whisper reads the whole window even for a short utterance: trimmed to
    /// the audio it loops on short clips, and trimmed to 10 s it adds words.
    pub(super) fn input(&self, audio: &[f32]) -> candle_core::Result<Tensor> {
        let device = self.cos.device();
        let audio = &audio[..audio.len().min(m::N_SAMPLES)];
        let frames = audio.len().div_ceil(m::HOP_LENGTH).min(m::N_FRAMES);
        let mut parts = Vec::with_capacity(2);
        if frames > 0 {
            parts.push(self.heard(audio, frames)?);
        }
        if frames < m::N_FRAMES {
            parts.push(Tensor::full(
                SILENCE,
                (m::N_FRAMES - frames, self.n_mels),
                device,
            )?);
        }
        let mel = Tensor::cat(&parts, 0)?;
        let max = mel.flatten_all()?.max(0)?.to_scalar::<f32>()?;
        mel.maximum(max - 8.0)?
            .affine(0.25, 1.0)?
            .t()?
            .contiguous()?
            .unsqueeze(0)
    }

    /// `log10` of the mel power of the first `frames` frames of `audio`:
    /// `(frames, n_mels)`.
    fn heard(&self, audio: &[f32], frames: usize) -> candle_core::Result<Tensor> {
        let windowed: Vec<f32> = (0..frames)
            .flat_map(|i| {
                self.hann
                    .iter()
                    .enumerate()
                    .map(move |(n, w)| audio.get(i * m::HOP_LENGTH + n).map_or(0.0, |&s| s * w))
            })
            .collect();
        let x = Tensor::from_vec(windowed, (frames, m::N_FFT), self.cos.device())?;
        let power = (x.matmul(&self.cos)?.sqr()? + x.matmul(&self.sin)?.sqr()?)?;
        power
            .matmul(&self.filters)?
            .clamp(1e-10f32, f32::MAX)?
            .log()?
            .affine(1.0 / std::f64::consts::LN_10, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The log-mel of frame `i` computed directly, before Whisper's clamp.
    fn direct(audio: &[f32], i: usize, filters: &[f32]) -> Vec<f32> {
        let power: Vec<f32> = (0..BINS)
            .map(|k| {
                let (mut re, mut im) = (0.0f64, 0.0f64);
                for n in 0..m::N_FFT {
                    let w = 0.5 - 0.5 * (std::f64::consts::TAU * n as f64 / m::N_FFT as f64).cos();
                    let s = f64::from(audio.get(i * m::HOP_LENGTH + n).copied().unwrap_or(0.0)) * w;
                    let a = std::f64::consts::TAU * (n * k) as f64 / m::N_FFT as f64;
                    re += s * a.cos();
                    im -= s * a.sin();
                }
                (re * re + im * im) as f32
            })
            .collect();
        filters
            .chunks(BINS)
            .map(|row| {
                row.iter()
                    .zip(&power)
                    .map(|(w, p)| w * p)
                    .sum::<f32>()
                    .max(1e-10)
                    .log10()
            })
            .collect()
    }

    #[test]
    fn the_log_mel_matches_a_direct_transform() {
        let audio: Vec<f32> = (0..16_000)
            .map(|i| (i as f32 * 0.07).sin() * 0.3 + (i as f32 * 0.011).sin() * 0.2)
            .collect();
        let mel = Mel::new(80, &Device::Cpu).expect("bases");
        let input = mel.input(&audio).expect("log-mel");
        assert_eq!(input.dims(), [1, 80, m::N_FRAMES]);
        for edge in [&[][..], &[0.1; m::N_SAMPLES + 5][..]] {
            assert_eq!(mel.input(edge).expect("edge").dims(), [1, 80, m::N_FRAMES]);
        }
        let got: Vec<Vec<f32>> = input
            .squeeze(0)
            .and_then(|t| t.t()?.to_vec2())
            .expect("values");
        let filters = filters(80);
        let max = (0..100)
            .flat_map(|i| direct(&audio, i, &filters))
            .fold(SILENCE, f32::max);
        for i in [0, 37, 99, 100, 2999] {
            for (j, &want) in direct(&audio, i, &filters).iter().enumerate() {
                let want = want.max(max - 8.0) / 4.0 + 1.0;
                assert!(
                    (got[i][j] - want).abs() < 1e-3,
                    "frame {i} band {j}: {} vs {want}",
                    got[i][j]
                );
            }
        }
    }

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
