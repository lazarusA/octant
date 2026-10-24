//! OpenAI's Whisper on candle: the model from `candle-transformers`, our
//! own log-mel (`mel.rs`), vocabulary (`tokens.rs`) and greedy decoding in
//! the utterance's own language, detected each time. Loaded from the bytes
//! of its files (`models.rs`): on the desktop from its data folder, on the
//! Apple GPU when there is one; in the browser's voice worker on the CPU.

mod bpe;
#[cfg(not(target_arch = "wasm32"))]
mod download;
mod mel;
pub mod models;
mod network;
mod tokens;

pub use models::megabytes;

use crate::voice::stt::WindowModel;
#[cfg(not(target_arch = "wasm32"))]
use crate::voice::stt::{Prep, SttEvent, WhisperSize};
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::ops::log_softmax;
use candle_transformers::models::whisper::{self as m, Config};
use tokens::Tokens;

/// The most prompt tokens kept before the text.
const PROMPT_TOKENS: usize = 64;

pub struct Whisper {
    model: network::Network,
    tokens: Tokens,
    mel: mel::Mel,
    /// `-inf` for tokens never written: the config's and every special
    /// token but the end of text.
    suppress: Tensor,
    device: Device,
    /// The text before what is transcribed (`set_prompt`): names the
    /// speaker may say, so they are written as spelled.
    prompt: Vec<u32>,
}

/// Downloads `size` if needed and loads it; what [`crate::voice::stt`]
/// runs on the provider's thread. Once tiny (the default) is ready, base is
/// downloaded in the background for computers that run it well.
#[cfg(not(target_arch = "wasm32"))]
pub fn prepare(size: WhisperSize, prep: &Prep) -> Result<Whisper, String> {
    prep.report(SttEvent::Preparing(Some(0.0)));
    let progress = |done| prep.report(SttEvent::Preparing(Some(done)));
    let dir = download::ensure(size, &progress, &|| prep.cancelled())?;
    prep.report(SttEvent::Preparing(None));
    let read = |file: &str| std::fs::read(dir.join(file)).map_err(|e| format!("{file}: {e}"));
    let model = Whisper::from_bytes(
        &read("config.json")?,
        &read("tokenizer.json")?,
        read(models::WEIGHTS)?,
    )?;
    if size == WhisperSize::Tiny {
        download::prefetch(WhisperSize::Base);
    }
    Ok(model)
}

/// The Apple GPU when there is one, else the CPU.
fn device() -> Device {
    #[cfg(target_os = "macos")]
    if let Ok(metal) = Device::new_metal(0) {
        return metal;
    }
    Device::Cpu
}

impl Whisper {
    /// The model from the bytes of its `config.json`, `tokenizer.json` and
    /// weights.
    pub fn from_bytes(config: &[u8], tokenizer: &[u8], weights: Vec<u8>) -> Result<Self, String> {
        let device = device();
        let quantize = device.is_cpu();
        Self::build(config, tokenizer, weights, device, quantize)
    }

    fn build(
        config: &[u8],
        tokenizer: &[u8],
        weights: Vec<u8>,
        device: Device,
        quantize: bool,
    ) -> Result<Self, String> {
        let config: Config = serde_json::from_slice(config).map_err(|e| e.to_string())?;
        let tokens = Tokens::parse(tokenizer)?;
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
        // The log-mel is quicker on the CPU than as many small GPU calls.
        let mel = mel::Mel::new(config.num_mel_bins, &Device::Cpu).map_err(|e| e.to_string())?;
        let model = network::Network::load_as(config, weights, &device, quantize)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            model,
            tokens,
            mel,
            suppress,
            device,
            prompt: Vec::new(),
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
        let ys = self.model.decode(&input, features, first)?;
        let (_, len, _) = ys.dims3()?;
        let last = self
            .model
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
        let features = self
            .model
            .encode(&self.mel.input(audio)?.to_device(&self.device)?)?;
        let language = self.detect_language(&features)?;
        let t = &self.tokens;
        let mut ids = Vec::new();
        if !self.prompt.is_empty() {
            ids.push(t.start_of_prev);
            ids.extend(&self.prompt);
        }
        let sot_at = ids.len();
        ids.extend([t.sot, language, t.transcribe, t.no_timestamps]);
        let (prompt, eot, no_speech_token) = (ids.len(), t.eot, t.no_speech);
        let (mut sum_logprob, mut no_speech) = (0.0f64, 0.0f64);
        let max_tokens = self.model.config().max_target_positions;
        for i in 0..(max_tokens / 2).min(max_tokens - prompt) {
            let (ys, logits) = self.next_logits(&ids, &features, i == 0)?;
            if i == 0 {
                // The chance of no speech is read at the start of transcript.
                // Only that row is projected onto the vocabulary, not the prompt's.
                let row = ys.i((..1, sot_at..sot_at + 1))?;
                let first = self.model.final_linear(&row)?.i(0)?.i(0)?;
                no_speech = candle_nn::ops::softmax(&first, 0)?
                    .i(no_speech_token as usize)?
                    .to_scalar::<f32>()? as f64;
            }
            let logits = logits.broadcast_add(&self.suppress)?;
            let next = logits.argmax(0)?.to_scalar::<u32>()?;
            if next == eot {
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

    /// Keeps the prompt's first tokens, at most `PROMPT_TOKENS`, cut after
    /// the last whole comma-separated name (the prompt lists names, the
    /// first the most wanted): candle's decoder caches no self-attention,
    /// so every token written reads the whole prompt again (600 characters
    /// of names made tiny 2.2x slower). An empty prompt clears it.
    fn set_prompt(&mut self, prompt: &str) {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            self.prompt.clear();
            return;
        }
        let mut tokens = self.tokens.encode(&format!(" {prompt}"));
        let keep = PROMPT_TOKENS.min(self.model.config().max_target_positions / 2 - 1);
        if tokens.len() > keep {
            let comma = self.tokens.encode(",").first().copied();
            let cut = tokens[..=keep]
                .iter()
                .rposition(|&t| Some(t) == comma)
                .unwrap_or(keep);
            tokens.truncate(cut);
        }
        self.prompt = tokens;
    }
}

#[cfg(test)]
mod tests;
