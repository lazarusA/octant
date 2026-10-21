//! Decode and predictor registry for GeoTIFF datasets.

pub mod decoders;
#[cfg(test)]
mod decoders_tests;
pub mod predictors;
#[cfg(test)]
mod predictors_tests;

use async_tiff::decoder::DecoderRegistry;
use async_tiff::tags::Compression;

pub use decoders::{DeflateDecoder, JpegDecoder, PackBitsDecoder, RobustLzwDecoder, ZstdDecoder};
pub use predictors::unpredict_buffer;

/// Create a DecoderRegistry populated with standard + robust decoders.
pub fn create_decoder_registry() -> DecoderRegistry {
    let mut registry = DecoderRegistry::default();
    let map = registry.as_mut();
    map.insert(Compression::PackBits, Box::new(PackBitsDecoder));
    map.insert(Compression::LZW, Box::new(RobustLzwDecoder));
    // Ours, not async-tiff's: capped, so a small tile never expands to gigabytes.
    map.insert(Compression::Deflate, Box::new(DeflateDecoder));
    map.insert(Compression::OldDeflate, Box::new(DeflateDecoder));
    map.insert(Compression::ZSTD, Box::new(ZstdDecoder));
    map.insert(Compression::JPEG, Box::new(JpegDecoder));
    map.insert(Compression::ModernJPEG, Box::new(JpegDecoder));
    registry
}
