//! Microphone samples at their own rate to the 16 kHz speech to text reads.
//! Each output sample averages the input span it covers, which also filters
//! out most of what would alias.
// shortcut: a box filter, not a proper low-pass; use a windowed sinc if
// transcription suffers from it.

use super::stt::SAMPLE_RATE;

pub struct Resampler {
    /// Input samples per output sample.
    step: f64,
    /// Where the next output sample starts in `pending`.
    pos: f64,
    pending: Vec<f32>,
}

impl Resampler {
    pub fn new(from: u32) -> Self {
        Self {
            step: f64::from(from.max(1)) / f64::from(SAMPLE_RATE),
            pos: 0.0,
            pending: Vec::new(),
        }
    }

    /// Appends to `out` the 16 kHz samples `input` completes.
    pub fn push(&mut self, input: &[f32], out: &mut Vec<f32>) {
        self.pending.extend_from_slice(input);
        while self.pos + self.step <= self.pending.len() as f64 {
            let (a, b) = (self.pos, self.pos + self.step);
            let sum: f64 = (a.floor() as usize..b.ceil() as usize)
                .map(|i| {
                    let covered = (b.min(i as f64 + 1.0) - a.max(i as f64)).max(0.0);
                    f64::from(self.pending[i]) * covered
                })
                .sum();
            out.push((sum / self.step) as f32);
            self.pos = b;
        }
        let used = self.pos.floor() as usize;
        self.pending.drain(..used);
        self.pos -= used as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    fn sine(rate: u32, freq: f32, len: usize) -> Vec<f32> {
        (0..len)
            .map(|i| (TAU * freq * i as f32 / rate as f32).sin())
            .collect()
    }

    #[test]
    fn rates_shrink_by_their_ratio_and_keep_a_tone() {
        for rate in [48_000, 44_100, 16_000] {
            let mut out = Vec::new();
            Resampler::new(rate).push(&sine(rate, 440.0, rate as usize), &mut out);
            let expected = SAMPLE_RATE as usize;
            assert!(out.len().abs_diff(expected) <= 1, "{rate}: {}", out.len());
            let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            assert!((0.95..=1.0).contains(&peak), "{rate}: peak {peak}");
        }
    }

    #[test]
    fn pieces_give_what_the_whole_gives() {
        let input = sine(44_100, 300.0, 9_000);
        let (mut whole, mut pieces) = (Vec::new(), Vec::new());
        Resampler::new(44_100).push(&input, &mut whole);
        let mut resampler = Resampler::new(44_100);
        for chunk in input.chunks(1_000) {
            resampler.push(chunk, &mut pieces);
        }
        assert_eq!(whole.len(), pieces.len());
        assert!(whole.iter().zip(&pieces).all(|(a, b)| (a - b).abs() < 1e-5));
    }
}
