//! Decode and predictor registry for GeoTIFF datasets.

pub mod decoders;
pub mod predictors;
#[cfg(test)]
mod predictors_tests;

use async_tiff::decoder::DecoderRegistry;
use async_tiff::tags::Compression;

pub use decoders::{JpegDecoder, PackBitsDecoder, RobustLzwDecoder};
pub use predictors::unpredict_buffer;

/// Create a DecoderRegistry populated with standard + robust decoders.
pub fn create_decoder_registry() -> DecoderRegistry {
    let mut registry = DecoderRegistry::default();
    let map = registry.as_mut();
    map.insert(Compression::PackBits, Box::new(PackBitsDecoder));
    map.insert(Compression::LZW, Box::new(RobustLzwDecoder));
    map.insert(Compression::JPEG, Box::new(JpegDecoder));
    map.insert(Compression::ModernJPEG, Box::new(JpegDecoder));
    registry
}
