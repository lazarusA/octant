//! The microphone while the voice bar listens, and what is made from it.

use crate::i18n::{PlaybackKey, tr};
use crate::ui::toast::{Severity, report};
use crate::voice::Capture;
use crate::voice::resample::Resampler;
use crate::voice::spectrum::Spectrum;

pub(super) struct Listening {
    pub(super) capture: Capture,
    /// Made once the microphone's sample rate is known.
    pub(super) spectrum: Option<Spectrum>,
    pub(super) resampler: Option<Resampler>,
    pub(super) samples: Vec<f32>,
    pub(super) resampled: Vec<f32>,
    /// Exact zeros heard so far, until a sound or the warning ends the check.
    silent: Option<usize>,
}

impl Listening {
    pub(super) fn new(capture: Capture) -> Self {
        Self {
            capture,
            spectrum: None,
            resampler: None,
            samples: Vec::new(),
            resampled: Vec::new(),
            silent: Some(0),
        }
    }

    /// Warns once when the first second at `rate` is exact zeros.
    pub(super) fn check_silence(&mut self, rate: u32) {
        let Some(silent) = &mut self.silent else {
            return;
        };
        *silent += self.samples.len();
        if self.samples.iter().any(|&s| s != 0.0) {
            self.silent = None;
        } else if rate > 0 && *silent >= rate as usize {
            // A microphone without permission (macOS) records exact zeros.
            report(
                Severity::Warning,
                tr(PlaybackKey::VoiceSilent),
                tr(PlaybackKey::VoiceSilentHint),
            );
            self.silent = None;
        }
    }
}
