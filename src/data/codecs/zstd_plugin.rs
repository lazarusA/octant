//! Pure Rust Zstd codec plugin for `zarrs` WebAssembly data decoding using `ruzstd`.

use zarrs::array::{ArrayBytesRaw, BytesRepresentation, CodecError};

pub const IDENTIFIER: &str = "zstd";

/// Pure Rust Zstandard codec using `ruzstd`.
#[derive(Clone, Debug, Default)]
pub struct RuzstdCodec;

impl RuzstdCodec {
    pub fn new() -> Self {
        Self
    }

    fn decode_bytes<'a>(
        value: ArrayBytesRaw<'a>,
        decoded_representation: &BytesRepresentation,
    ) -> Result<ArrayBytesRaw<'a>, CodecError> {
        let decoder = ruzstd::decoding::StreamingDecoder::new(&value[..]).map_err(|e| {
            CodecError::Other(format!("Ruzstd streaming decoder init failed: {e:?}"))
        })?;
        // Never more than the chunk's own size: a small chunk expanding to
        // gigabytes would run the page out of memory.
        let (cap, reserve) = (
            super::capped::cap_for(decoded_representation),
            super::capped::reserve_for(decoded_representation),
        );
        let out = super::capped::read_capped(decoder, cap, reserve)
            .map_err(|e| CodecError::Other(format!("Ruzstd decompression failed: {e}")))?;
        Ok(out.into())
    }

    fn encode_bytes(_value: ArrayBytesRaw<'_>) -> Result<ArrayBytesRaw<'_>, CodecError> {
        Err(CodecError::Other(
            "Encoding is not supported for WASM zstd codec".to_string(),
        ))
    }
}

super::plugin::bytes_codec_plugin!(
    RuzstdCodec,
    IDENTIFIER,
    "numcodecs.zstd",
    {"level": 0, "checksum": false}
);
