//! OpenAI's Whisper on candle: the model from `candle-transformers`, our
//! own mel filters (`mel.rs`), vocabulary (`tokens.rs`) and greedy decoding
//! in the utterance's own language, detected each time. Runs on the Apple
//! GPU when there is one, else the CPU.

mod download;
mod mel;
mod tokens;

pub use download::megabytes;

use crate::voice::stt::{Prep, SttEvent, WhisperSize, WindowModel};
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_nn::ops::log_softmax;
use candle_transformers::models::whisper::{self as m, Config};
use std::path::Path;
use tokens::Tokens;

pub struct Whisper {
    model: m::model::Whisper,
    tokens: Tokens,
    filters: Vec<f32>,
    /// `-inf` for tokens never written: the config's and every special
    /// token but the end of text.
    suppress: Tensor,
    device: Device,
}

/// Downloads `size` if needed and loads it; what [`crate::voice::stt`]
/// runs on the provider's thread.
pub fn prepare(size: WhisperSize, prep: &Prep) -> Result<Whisper, String> {
    prep.report(SttEvent::Preparing(Some(0.0)));
    let dir = download::ensure(size, prep)?;
    prep.report(SttEvent::Preparing(None));
    Whisper::load(&dir).map_err(|e| e.to_string())
}

fn device() -> Device {
    #[cfg(target_os = "macos")]
    if let Ok(metal) = Device::new_metal(0) {
        return metal;
    }
    Device::Cpu
}

impl Whisper {
    fn load(dir: &Path) -> Result<Self, String> {
        let read = |file: &str| std::fs::read(dir.join(file)).map_err(|e| format!("{file}: {e}"));
        let config: Config =
            serde_json::from_slice(&read("config.json")?).map_err(|e| e.to_string())?;
        let tokens = Tokens::parse(&read("tokenizer.json")?)?;
        let device = device();
        let vb =
            VarBuilder::from_buffered_safetensors(read("model.safetensors")?, m::DTYPE, &device)
                .map_err(|e| e.to_string())?;
        let suppress: Vec<f32> = (0..config.vocab_size as u32)
            .map(|id| {
                if id > tokens.eot || config.suppress_tokens.contains(&id) {
                    f32::NEG_INFINITY
                } else {
                    0.0
                }
            })
            .collect();
        let suppress = Tensor::new(suppress.as_slice(), &device).map_err(|e| e.to_string())?;
        let filters = mel::filters(config.num_mel_bins);
        let model = m::model::Whisper::load(&vb, config).map_err(|e| e.to_string())?;
        Ok(Self {
            model,
            tokens,
            filters,
            suppress,
            device,
        })
    }

    /// The decoder's scores for the token after `ids`.
    fn next_logits(
        &mut self,
        ids: &[u32],
        features: &Tensor,
        first: bool,
    ) -> candle_core::Result<(Tensor, Tensor)> {
        let input = Tensor::new(ids, &self.device)?.unsqueeze(0)?;
        let ys = self.model.decoder.forward(&input, features, first)?;
        let (_, len, _) = ys.dims3()?;
        let last = self
            .model
            .decoder
            .final_linear(&ys.i((..1, len - 1..))?)?
            .i(0)?
            .i(0)?;
        Ok((ys, last))
    }

    /// The most likely of Whisper's languages for the audio.
    fn detect_language(&mut self, features: &Tensor) -> candle_core::Result<u32> {
        let (_, logits) = self.next_logits(&[self.tokens.sot], features, true)?;
        let scores: Vec<f32> = logits.to_vec1()?;
        let languages = self.tokens.languages.clone();
        Ok(languages
            .max_by(|&a, &b| scores[a as usize].total_cmp(&scores[b as usize]))
            .unwrap_or(self.tokens.sot + 1))
    }

    fn run(&mut self, audio: &[f32]) -> candle_core::Result<String> {
        let mel = mel::input(&self.model.config, &self.filters, audio, &self.device)?;
        let features = self.model.encoder.forward(&mel, true)?;
        let language = self.detect_language(&features)?;
        let t = &self.tokens;
        let mut ids = vec![t.sot, language, t.transcribe, t.no_timestamps];
        let (prompt, eot, no_speech_token) = (ids.len(), t.eot, t.no_speech);
        let (mut sum_logprob, mut no_speech) = (0.0f64, 0.0f64);
        for i in 0..self.model.config.max_target_positions / 2 {
            let (ys, logits) = self.next_logits(&ids, &features, i == 0)?;
            if i == 0 {
                // The chance of no speech is read where the text would start.
                let first = self.model.decoder.final_linear(&ys.i(..1)?)?.i(0)?.i(0)?;
                no_speech = candle_nn::ops::softmax(&first, 0)?
                    .i(no_speech_token as usize)?
                    .to_scalar::<f32>()? as f64;
            }
            let logits = logits.broadcast_add(&self.suppress)?;
            let next = logits.argmax(0)?.to_scalar::<u32>()?;
            if next == eot || ids.len() >= self.model.config.max_target_positions {
                break;
            }
            sum_logprob += log_softmax(&logits, 0)?
                .i(next as usize)?
                .to_scalar::<f32>()? as f64;
            ids.push(next);
        }
        let avg_logprob = sum_logprob / (ids.len() - prompt).max(1) as f64;
        if no_speech > m::NO_SPEECH_THRESHOLD && avg_logprob < m::LOGPROB_THRESHOLD {
            return Ok(String::new());
        }
        Ok(self.tokens.decode(&ids[prompt..]))
    }
}

impl WindowModel for Whisper {
    fn transcribe(&mut self, audio: &[f32]) -> Result<String, String> {
        self.run(audio).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests;
