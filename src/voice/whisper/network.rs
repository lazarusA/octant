//! The Whisper network: full precision on a GPU, quantized to 8 bits on the
//! CPU (the browser's voice worker, desktops without one), where it runs
//! faster in less memory. Quantizing happens at load, from OpenAI's own
//! weights, so only their files are ever downloaded: each matrix whose rows
//! split into 32-value blocks becomes Q8_0, the rest stays f32, and candle
//! reads the result as an in-memory GGUF file.

use candle_core::quantized::{GgmlDType, QTensor, gguf_file};
use candle_core::{Device, Result, Tensor};
use candle_transformers::models::whisper::{self as m, Config, quantized_model};
use candle_transformers::quantized_var_builder::VarBuilder as QuantizedVarBuilder;

pub(super) enum Network {
    Full(m::model::Whisper),
    Quantized(quantized_model::Whisper),
}

impl Network {
    /// Loads `weights` (safetensors) for `device`, quantized or not.
    pub(super) fn load_as(
        config: Config,
        weights: Vec<u8>,
        device: &Device,
        quantize: bool,
    ) -> Result<Self> {
        if quantize {
            let vb = quantized(&weights, device)?;
            return Ok(Self::Quantized(quantized_model::Whisper::load(
                &vb, config,
            )?));
        }
        let vb = candle_nn::VarBuilder::from_buffered_safetensors(weights, m::DTYPE, device)?;
        Ok(Self::Full(m::model::Whisper::load(&vb, config)?))
    }

    pub(super) fn config(&self) -> &Config {
        match self {
            Self::Full(w) => &w.config,
            Self::Quantized(w) => &w.config,
        }
    }

    pub(super) fn encode(&mut self, mel: &Tensor) -> Result<Tensor> {
        match self {
            Self::Full(w) => w.encoder.forward(mel, true),
            Self::Quantized(w) => w.encoder.forward(mel, true),
        }
    }

    /// The decoder's hidden states for `ids`; `first` starts a new text.
    pub(super) fn decode(
        &mut self,
        ids: &Tensor,
        features: &Tensor,
        first: bool,
    ) -> Result<Tensor> {
        match self {
            Self::Full(w) => w.decoder.forward(ids, features, first),
            Self::Quantized(w) => w.decoder.forward(ids, features, first),
        }
    }

    pub(super) fn final_linear(&self, x: &Tensor) -> Result<Tensor> {
        match self {
            Self::Full(w) => w.decoder.final_linear(x),
            Self::Quantized(w) => w.decoder.final_linear(x),
        }
    }
}

/// `weights` quantized, as a quantized model reads them.
fn quantized(weights: &[u8], device: &Device) -> Result<QuantizedVarBuilder> {
    let tensors = candle_core::safetensors::load_buffer(weights, &Device::Cpu)?;
    let mut quantized = Vec::with_capacity(tensors.len());
    for (name, tensor) in tensors {
        let blocks = tensor.rank() >= 2 && tensor.dims().last().is_some_and(|d| d % 32 == 0);
        let dtype = if blocks {
            GgmlDType::Q8_0
        } else {
            GgmlDType::F32
        };
        quantized.push((name, QTensor::quantize(&tensor.to_dtype(m::DTYPE)?, dtype)?));
    }
    let refs: Vec<(&str, &QTensor)> = quantized.iter().map(|(n, t)| (n.as_str(), t)).collect();
    let mut gguf = std::io::Cursor::new(Vec::new());
    gguf_file::write(&mut gguf, &[], &refs)?;
    QuantizedVarBuilder::from_gguf_buffer(gguf.get_ref(), device)
}
