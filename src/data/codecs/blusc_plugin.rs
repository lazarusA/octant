//! Pure Rust Blosc codec plugin for `zarrs` WebAssembly data decoding.

use zarrs::array::{ArrayBytesRaw, BytesRepresentation, CodecError};

pub const IDENTIFIER: &str = "blosc";

/// Pure Rust Blosc codec using `blusc`.
#[derive(Clone, Debug, Default)]
pub struct BluscCodec;

impl BluscCodec {
    pub fn new() -> Self {
        Self
    }

    fn decode_bytes<'a>(
        value: ArrayBytesRaw<'a>,
        decoded_representation: &BytesRepresentation,
    ) -> Result<ArrayBytesRaw<'a>, CodecError> {
        // Blosc allocates the size its header claims before decoding: never
        // more than the chunk's own size.
        let (claimed, _, _) = blusc::api::blosc2_cbuffer_sizes(&value);
        let cap = super::capped::cap_for(decoded_representation);
        if claimed > cap {
            return Err(CodecError::Other(format!(
                "Blosc chunk claims {claimed} bytes, more than its {cap}"
            )));
        }
        let decompressed = blusc::convenience::blosc2_decompress(&value)
            .map_err(|e| CodecError::Other(format!("Blusc decompression failed: {e}")))?;
        Ok(decompressed.into())
    }

    fn encode_bytes(value: ArrayBytesRaw<'_>) -> Result<ArrayBytesRaw<'_>, CodecError> {
        let compressed = blusc::convenience::blosc2_compress(&value)
            .map_err(|e| CodecError::Other(format!("Blusc compression failed: {e:?}")))?;
        Ok(compressed.into())
    }
}

super::plugin::bytes_codec_plugin!(
    BluscCodec,
    IDENTIFIER,
    "numcodecs.blosc",
    {"cname": "lz4", "clevel": 5, "shuffle": "noshuffle", "blocksize": 0}
);
