//! Cuts 16 kHz audio into utterances: speech starts one (with a moment of
//! the audio before it, so the first sound is whole), and a pause or the
//! length a model reads at once ends it. While one is under way, it asks
//! for a partial text every [`PARTIAL_EVERY`] samples.

use super::SAMPLE_RATE;

const SECOND: usize = SAMPLE_RATE as usize;
/// 10 ms frames, each loud or quiet.
const FRAME: usize = SECOND / 100;
/// A frame louder than this (about -40 dBFS) is speech.
// shortcut: a fixed threshold; adapt it to the room (as the voice bar's
// disk does) or use a voice activity model if it cuts words or hears fans.
const SPEECH_RMS: f32 = 0.01;
const PRE_ROLL: usize = SECOND * 3 / 10;
const END_QUIET: usize = SECOND * 8 / 10;
const PARTIAL_EVERY: usize = SECOND / 2;
/// Whisper reads at most 30 s at once.
const MAX_UTTERANCE: usize = SECOND * 28;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Step {
    Wait,
    /// Transcribe [`Segmenter::audio`] for a partial text.
    Partial,
    /// The utterance ended: transcribe [`Segmenter::take`] for its final text.
    Final,
}

#[derive(Default)]
pub(super) struct Segmenter {
    audio: Vec<f32>,
    heard: bool,
    /// Quiet samples since the last speech.
    quiet: usize,
    since_partial: usize,
    /// The part of a frame the last call left over.
    partial_frame: usize,
}

impl Segmenter {
    /// Adds `samples`; stops at the first sample that ends an utterance, so
    /// the rest (returned as the count consumed) starts the next one.
    pub(super) fn feed(&mut self, samples: &[f32]) -> (Step, usize) {
        let mut step = Step::Wait;
        let mut used = 0;
        while used < samples.len() {
            let take = (FRAME - self.partial_frame).min(samples.len() - used);
            let frame = &samples[used..used + take];
            used += take;
            self.audio.extend_from_slice(frame);
            self.partial_frame = (self.partial_frame + take) % FRAME;

            let rms = (frame.iter().map(|s| s * s).sum::<f32>() / take as f32).sqrt();
            if rms > SPEECH_RMS {
                self.heard = true;
                self.quiet = 0;
            } else {
                self.quiet += take;
            }
            if !self.heard {
                let excess = self.audio.len().saturating_sub(PRE_ROLL);
                self.audio.drain(..excess);
                continue;
            }
            self.since_partial += take;
            if self.quiet >= END_QUIET || self.audio.len() >= MAX_UTTERANCE {
                return (Step::Final, used);
            }
            if self.since_partial >= PARTIAL_EVERY {
                self.since_partial = 0;
                step = Step::Partial;
            }
        }
        (step, used)
    }

    /// The speaker stopped: a final text if anything was said.
    pub(super) fn finish(&self) -> Step {
        if self.heard { Step::Final } else { Step::Wait }
    }

    pub(super) fn audio(&self) -> &[f32] {
        &self.audio
    }

    /// The utterance's audio, starting the next one empty.
    pub(super) fn take(&mut self) -> Vec<f32> {
        let audio = std::mem::take(&mut self.audio);
        *self = Self::default();
        audio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEECH: [f32; FRAME] = [0.3; FRAME];
    const QUIET: [f32; FRAME] = [0.0; FRAME];

    /// Feeds `frames` 10 ms frames of `frame`; the steps that were not Wait.
    fn run(seg: &mut Segmenter, frame: &[f32], frames: usize) -> Vec<Step> {
        (0..frames)
            .map(|_| seg.feed(frame).0)
            .filter(|&s| s != Step::Wait)
            .collect()
    }

    #[test]
    fn silence_asks_nothing_and_keeps_only_the_pre_roll() {
        let mut seg = Segmenter::default();
        assert!(run(&mut seg, &QUIET, 500).is_empty());
        assert_eq!(seg.audio().len(), PRE_ROLL);
        assert_eq!(seg.finish(), Step::Wait);
    }

    #[test]
    fn speech_then_a_pause_gives_partials_then_a_final() {
        let mut seg = Segmenter::default();
        run(&mut seg, &QUIET, 100);
        // One second of speech: a partial every half second.
        assert_eq!(run(&mut seg, &SPEECH, 100), [Step::Partial, Step::Partial]);
        let steps = run(&mut seg, &QUIET, 80);
        assert_eq!(steps.last(), Some(&Step::Final));
        let audio = seg.take();
        assert_eq!(audio.len(), PRE_ROLL + SECOND + END_QUIET);
        assert_eq!(seg.finish(), Step::Wait, "the next utterance starts empty");
    }

    #[test]
    fn a_long_utterance_ends_at_the_model_length() {
        let mut seg = Segmenter::default();
        let steps = run(&mut seg, &SPEECH, MAX_UTTERANCE / FRAME);
        assert_eq!(steps.last(), Some(&Step::Final));
    }

    #[test]
    fn the_rest_of_a_chunk_after_a_final_is_left_over() {
        let mut seg = Segmenter::default();
        seg.feed(&[0.3; SECOND]);
        let quiet = vec![0.0; END_QUIET + 1_000];
        let (step, used) = seg.feed(&quiet);
        assert_eq!((step, used), (Step::Final, END_QUIET));
    }

    #[test]
    fn finishing_mid_speech_gives_a_final() {
        let mut seg = Segmenter::default();
        run(&mut seg, &SPEECH, 10);
        assert_eq!(seg.finish(), Step::Final);
    }
}
