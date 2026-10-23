//! Mel spectrogram columns from mono samples at any rate: 1024-sample Hann
//! windows every 10 ms, folded into [`BANDS`] mel bands from 50 Hz to 8 kHz
//! (or the rate's Nyquist), each a 0..1 position on a dB scale.

use std::f32::consts::PI;

/// Mel bands per column, lowest first.
pub const BANDS: usize = 64;
/// Columns per second (one every 10 ms).
pub const COLUMNS_PER_SECOND: u32 = 100;

const FFT: usize = 1024;
const LOW_HZ: f32 = 50.0;
const HIGH_HZ: f32 = 8000.0;
/// dB range mapped onto 0..1; a full-scale sine reads about 0 dB.
const FLOOR_DB: f32 = -90.0;
const TOP_DB: f32 = -10.0;

pub struct Spectrum {
    hop: usize,
    window: Vec<f32>,
    /// Per band: the first FFT bin it reads and the weights from there on.
    filters: Vec<(usize, Vec<f32>)>,
    pending: Vec<f32>,
    re: Vec<f32>,
    im: Vec<f32>,
}

impl Spectrum {
    pub fn new(rate: u32) -> Self {
        let window = (0..FFT)
            .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f32 / FFT as f32).cos())
            .collect();
        Self {
            hop: (rate / COLUMNS_PER_SECOND).max(1) as usize,
            window,
            filters: mel_filters(rate as f32),
            pending: Vec::with_capacity(FFT * 2),
            re: vec![0.0; FFT],
            im: vec![0.0; FFT],
        }
    }

    /// Adds `samples` and calls `column` for each column they complete.
    pub fn push(&mut self, samples: &[f32], mut column: impl FnMut(&[f32; BANDS])) {
        self.pending.extend_from_slice(samples);
        let mut start = 0;
        while start + FFT <= self.pending.len() {
            let bands = self.column(start);
            column(&bands);
            start += self.hop;
        }
        self.pending.drain(..start);
    }

    fn column(&mut self, start: usize) -> [f32; BANDS] {
        let frame = &self.pending[start..start + FFT];
        for (i, (&s, &w)) in frame.iter().zip(&self.window).enumerate() {
            self.re[i] = s * w;
            self.im[i] = 0.0;
        }
        fft(&mut self.re, &mut self.im);
        // A full-scale sine under a Hann window peaks at FFT/4.
        let norm = (FFT as f32 / 4.0).powi(2);
        let mut bands = [0.0; BANDS];
        for (band, (first, weights)) in bands.iter_mut().zip(&self.filters) {
            let power: f32 = weights
                .iter()
                .enumerate()
                .map(|(k, w)| w * (self.re[first + k].powi(2) + self.im[first + k].powi(2)))
                .sum();
            let db = 10.0 * (power / norm + 1e-12).log10();
            *band = ((db - FLOOR_DB) / (TOP_DB - FLOOR_DB)).clamp(0.0, 1.0);
        }
        bands
    }
}

fn mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn hz(mel: f32) -> f32 {
    700.0 * (10f32.powf(mel / 2595.0) - 1.0)
}

/// Triangular mel filters over the FFT bins, each normalized to sum 1; a
/// band narrower than a bin reads its nearest bin.
fn mel_filters(rate: f32) -> Vec<(usize, Vec<f32>)> {
    let bin_hz = rate / FFT as f32;
    let (lo, hi) = (mel(LOW_HZ), mel(HIGH_HZ.min(rate / 2.0)));
    let edge = |i: usize| hz(lo + (hi - lo) * i as f32 / (BANDS + 1) as f32) / bin_hz;
    (0..BANDS)
        .map(|b| {
            let (left, center, right) = (edge(b), edge(b + 1), edge(b + 2));
            let first = left.ceil() as usize;
            let last = (right.floor() as usize).min(FFT / 2);
            let mut weights: Vec<f32> = (first..=last.max(first))
                .map(|k| {
                    let k = k as f32;
                    if k <= center {
                        (k - left) / (center - left)
                    } else {
                        (right - k) / (right - center)
                    }
                    .max(0.0)
                })
                .collect();
            let sum: f32 = weights.iter().sum();
            if sum <= f32::EPSILON {
                return ((center.round() as usize).min(FFT / 2), vec![1.0]);
            }
            weights.iter_mut().for_each(|w| *w /= sum);
            (first, weights)
        })
        .collect()
}

/// In-place radix-2 FFT; `re.len()` must be a power of two.
fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let angle = -2.0 * PI / len as f32;
        for start in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (s, c) = (angle * k as f32).sin_cos();
                let (a, b) = (start + k, start + k + len / 2);
                let (tr, ti) = (re[b] * c - im[b] * s, re[b] * s + im[b] * c);
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
            }
        }
        len <<= 1;
    }
}

#[cfg(test)]
#[path = "spectrum_tests.rs"]
mod tests;
