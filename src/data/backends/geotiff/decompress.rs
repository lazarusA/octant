//! Self-contained decompression routines integrated with `async-tiff`'s DecoderRegistry.

use std::io::Read;

use async_tiff::decoder::{Decoder, DecoderRegistry};
use async_tiff::error::{AsyncTiffError, AsyncTiffResult};
use async_tiff::tags::{Compression, PhotometricInterpretation};
use bytes::Bytes;

/// Construct a `DecoderRegistry` populated with non-panicking, robust decoders.
pub fn create_robust_decoder_registry() -> DecoderRegistry {
    let mut registry = DecoderRegistry::empty();
    let map = registry.as_mut();
    map.insert(Compression::None, Box::new(UncompressedDecoder));
    map.insert(Compression::PackBits, Box::new(PackBitsDecoder));
    map.insert(Compression::LZW, Box::new(RobustLzwDecoder));
    map.insert(Compression::Deflate, Box::new(DeflateDecoder));
    map.insert(Compression::OldDeflate, Box::new(DeflateDecoder));
    map.insert(Compression::ZSTD, Box::new(ZstdDecoder));
    map.insert(Compression::ModernJPEG, Box::new(JpegDecoder));
    map.insert(Compression::JPEG, Box::new(JpegDecoder));
    registry
}

/// Decompress raw compressed tile/strip bytes using the provided `DecoderRegistry`.
pub fn decompress_chunk(
    compressed: &[u8],
    compression: Compression,
    photometric: PhotometricInterpretation,
    jpeg_tables: Option<&[u8]>,
    samples: u16,
    bits: u16,
    registry: &DecoderRegistry,
) -> Result<Vec<u8>, String> {
    if let Some(decoder) = registry.as_ref().get(&compression) {
        let bytes = Bytes::copy_from_slice(compressed);
        decoder
            .decode_tile(bytes, photometric, jpeg_tables, samples, bits, None)
            .map_err(|e| e.to_string())
    } else {
        Err(format!("Unsupported TIFF compression: {compression:?}"))
    }
}

/// Passthrough uncompressed decoder.
#[derive(Debug, Clone)]
pub struct UncompressedDecoder;

impl Decoder for UncompressedDecoder {
    fn decode_tile(
        &self,
        buffer: Bytes,
        _photo: PhotometricInterpretation,
        _tables: Option<&[u8]>,
        _samples: u16,
        _bits: u16,
        _lerc: Option<&[u32]>,
    ) -> AsyncTiffResult<Vec<u8>> {
        Ok(buffer.to_vec())
    }
}

/// Apple/TIFF PackBits byte-run RLE decompressor.
#[derive(Debug, Clone)]
pub struct PackBitsDecoder;

impl Decoder for PackBitsDecoder {
    fn decode_tile(
        &self,
        buffer: Bytes,
        _photo: PhotometricInterpretation,
        _tables: Option<&[u8]>,
        _samples: u16,
        _bits: u16,
        _lerc: Option<&[u32]>,
    ) -> AsyncTiffResult<Vec<u8>> {
        let input = buffer.as_ref();
        let mut out = Vec::with_capacity(input.len() * 2);
        let mut i = 0;
        while i < input.len() {
            let n = input[i] as i8;
            i += 1;
            if n >= 0 {
                let count = n as usize + 1;
                if i + count > input.len() {
                    return Err(AsyncTiffError::General("PackBits run exceeds input".into()));
                }
                out.extend_from_slice(&input[i..i + count]);
                i += count;
            } else if n != -128 {
                let count = (-n) as usize + 1;
                if i >= input.len() {
                    return Err(AsyncTiffError::General("PackBits byte missing".into()));
                }
                let byte = input[i];
                i += 1;
                out.resize(out.len() + count, byte);
            }
        }
        Ok(out)
    }
}

/// LZW decompressor using weezl with TIFF MSB/LSB bit ordering and compat fallbacks.
#[derive(Debug, Clone)]
pub struct RobustLzwDecoder;

impl Decoder for RobustLzwDecoder {
    fn decode_tile(
        &self,
        buffer: Bytes,
        _photo: PhotometricInterpretation,
        _tables: Option<&[u8]>,
        _samples: u16,
        _bits: u16,
        _lerc: Option<&[u32]>,
    ) -> AsyncTiffResult<Vec<u8>> {
        let input = buffer.as_ref();
        // 1. Standard TIFF LZW (MSB with TIFF size switch)
        let mut decoder = weezl::decode::Decoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8);
        if let Ok(data) = decoder.decode(input) {
            return Ok(data);
        }
        // 2. Standard LZW (MSB standard switch)
        let mut compat = weezl::decode::Decoder::new(weezl::BitOrder::Msb, 8);
        if let Ok(data) = compat.decode(input) {
            return Ok(data);
        }
        // 3. LSB LZW (FillOrder=2 / compat mode with TIFF switch)
        let mut lsb = weezl::decode::Decoder::with_tiff_size_switch(weezl::BitOrder::Lsb, 8);
        if let Ok(data) = lsb.decode(input) {
            return Ok(data);
        }
        // 4. LSB LZW (FillOrder=2 / compat mode with standard switch)
        let mut lsb_std = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, 8);
        lsb_std
            .decode(input)
            .map_err(|e| AsyncTiffError::General(format!("LZW decompression failed: {e:?}")))
    }
}

/// Deflate / Zlib decompressor.
#[derive(Debug, Clone)]
pub struct DeflateDecoder;

impl Decoder for DeflateDecoder {
    fn decode_tile(
        &self,
        buffer: Bytes,
        _photo: PhotometricInterpretation,
        _tables: Option<&[u8]>,
        _samples: u16,
        _bits: u16,
        _lerc: Option<&[u32]>,
    ) -> AsyncTiffResult<Vec<u8>> {
        let mut decoder = flate2::read::ZlibDecoder::new(buffer.as_ref());
        let mut out = Vec::new();
        decoder
            .read_to_end(&mut out)
            .map_err(|e| AsyncTiffError::General(format!("Deflate failed: {e}")))?;
        Ok(out)
    }
}

/// Zstd decompressor using ruzstd.
#[derive(Debug, Clone)]
pub struct ZstdDecoder;

impl Decoder for ZstdDecoder {
    fn decode_tile(
        &self,
        buffer: Bytes,
        _photo: PhotometricInterpretation,
        _tables: Option<&[u8]>,
        _samples: u16,
        _bits: u16,
        _lerc: Option<&[u32]>,
    ) -> AsyncTiffResult<Vec<u8>> {
        let mut decoder = ruzstd::decoding::StreamingDecoder::new(buffer.as_ref())
            .map_err(|e| AsyncTiffError::General(format!("Zstd init failed: {e:?}")))?;
        let mut out = Vec::new();
        decoder
            .read_to_end(&mut out)
            .map_err(|e| AsyncTiffError::General(format!("Zstd decompression failed: {e:?}")))?;
        Ok(out)
    }
}

/// JPEG decompressor combining JPEGTables with tile/strip payload.
#[derive(Debug, Clone)]
pub struct JpegDecoder;

impl Decoder for JpegDecoder {
    fn decode_tile(
        &self,
        buffer: Bytes,
        _photo: PhotometricInterpretation,
        jpeg_tables: Option<&[u8]>,
        _samples: u16,
        _bits: u16,
        _lerc: Option<&[u32]>,
    ) -> AsyncTiffResult<Vec<u8>> {
        let input = buffer.as_ref();
        let combined_data = if let Some(tables) = jpeg_tables
            && tables.len() >= 4
            && input.len() >= 2
        {
            let mut combined = Vec::with_capacity(tables.len() + input.len());
            let tables_payload = if tables.ends_with(&[0xFF, 0xD9]) {
                &tables[..tables.len() - 2]
            } else {
                tables
            };
            let input_payload = if input.starts_with(&[0xFF, 0xD8]) {
                &input[2..]
            } else {
                input
            };
            combined.extend_from_slice(tables_payload);
            combined.extend_from_slice(input_payload);
            combined
        } else {
            input.to_vec()
        };

        let img = image::load_from_memory_with_format(&combined_data, image::ImageFormat::Jpeg)
            .map_err(|e| AsyncTiffError::General(format!("JPEG decode failed: {e}")))?;
        Ok(img.to_rgb8().into_raw())
    }
}
