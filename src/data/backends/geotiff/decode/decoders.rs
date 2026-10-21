//! Pure-Rust decoder implementations for TIFF compression formats.

use async_tiff::decoder::Decoder;
use async_tiff::error::{AsyncTiffError, AsyncTiffResult};
use async_tiff::tags::PhotometricInterpretation;
use bytes::Bytes;

use crate::data::codecs::capped::{CappedWriter, MAX_DECODED_BYTES, UP_FRONT_BYTES, read_capped};

/// A decoder whose output never exceeds [`MAX_DECODED_BYTES`]: a small tile
/// must not expand to gigabytes and run the page out of memory.
macro_rules! capped_decoder {
    ($(#[$doc:meta])* $name:ident, $decode:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone)]
        pub struct $name;

        impl Decoder for $name {
            fn decode_tile(
                &self,
                buf: Bytes,
                _p: PhotometricInterpretation,
                _t: Option<&[u8]>,
                _s: u16,
                _b: u16,
                _l: Option<&[u32]>,
            ) -> AsyncTiffResult<Vec<u8>> {
                $decode(&buf, MAX_DECODED_BYTES)
            }
        }
    };
}

capped_decoder!(PackBitsDecoder, packbits);
capped_decoder!(
    /// LZW in either bit order, with or without TIFF's early size switch.
    RobustLzwDecoder,
    lzw
);
capped_decoder!(DeflateDecoder, deflate);
capped_decoder!(ZstdDecoder, zstd);

fn too_large(e: impl std::fmt::Display) -> AsyncTiffError {
    AsyncTiffError::General(format!("TIFF tile: {e}"))
}

pub(super) fn packbits(input: &[u8], cap: usize) -> AsyncTiffResult<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len().saturating_mul(2).min(cap));
    let mut i = 0;
    while i < input.len() {
        let n = input[i] as i8;
        i += 1;
        // -128 is a no-op; n >= 0 copies n + 1 bytes, else repeats one -n + 1 times.
        let count = match n {
            -128 => continue,
            0.. => n as usize + 1,
            _ => n.unsigned_abs() as usize + 1,
        };
        if out.len() + count > cap {
            return Err(too_large(format!("decoded data exceeds {cap} bytes")));
        }
        if n >= 0 {
            let run = input
                .get(i..i + count)
                .ok_or_else(|| AsyncTiffError::General("PackBits run exceeds input".into()))?;
            out.extend_from_slice(run);
            i += count;
        } else {
            let &byte = input
                .get(i)
                .ok_or_else(|| AsyncTiffError::General("PackBits byte missing".into()))?;
            out.resize(out.len() + count, byte);
            i += 1;
        }
    }
    Ok(out)
}

pub(super) fn lzw(input: &[u8], cap: usize) -> AsyncTiffResult<Vec<u8>> {
    use weezl::{BitOrder, decode::Decoder as Lzw};
    let mut last = None;
    for decoder in [
        Lzw::with_tiff_size_switch(BitOrder::Msb, 8),
        Lzw::new(BitOrder::Msb, 8),
        Lzw::with_tiff_size_switch(BitOrder::Lsb, 8),
        Lzw::new(BitOrder::Lsb, 8),
    ] {
        let mut decoder = decoder;
        let mut out = CappedWriter::new(cap, input.len().saturating_mul(4));
        match decoder.into_stream(&mut out).decode_all(input).status {
            Ok(()) => return Ok(out.into_inner()),
            Err(e) => last = Some(e),
        }
    }
    let e = last.map_or_else(|| "no decoder".into(), |e| e.to_string());
    Err(AsyncTiffError::General(format!(
        "LZW decompression failed: {e}"
    )))
}

pub(super) fn deflate(input: &[u8], cap: usize) -> AsyncTiffResult<Vec<u8>> {
    read_capped(flate2::read::ZlibDecoder::new(input), cap, UP_FRONT_BYTES).map_err(too_large)
}

pub(super) fn zstd(input: &[u8], cap: usize) -> AsyncTiffResult<Vec<u8>> {
    let decoder = zstd::Decoder::new(input).map_err(too_large)?;
    read_capped(decoder, cap, UP_FRONT_BYTES).map_err(too_large)
}

#[derive(Debug, Clone)]
pub struct JpegDecoder;

impl Decoder for JpegDecoder {
    fn decode_tile(
        &self,
        buf: Bytes,
        _p: PhotometricInterpretation,
        tables: Option<&[u8]>,
        _s: u16,
        _b: u16,
        _l: Option<&[u32]>,
    ) -> AsyncTiffResult<Vec<u8>> {
        let input = buf.as_ref();
        let combined = if let Some(t) = tables
            && t.len() >= 4
            && input.len() >= 2
        {
            let mut c = Vec::with_capacity(t.len() + input.len());
            let t_body = if t.ends_with(&[0xFF, 0xD9]) {
                &t[..t.len() - 2]
            } else {
                t
            };
            let in_body = if input.starts_with(&[0xFF, 0xD8]) {
                &input[2..]
            } else {
                input
            };
            c.extend_from_slice(t_body);
            c.extend_from_slice(in_body);
            c
        } else {
            input.to_vec()
        };
        let img = image::load_from_memory_with_format(&combined, image::ImageFormat::Jpeg)
            .map_err(|e| AsyncTiffError::General(format!("JPEG decode failed: {e}")))?;
        Ok(img.to_rgb8().into_raw())
    }
}
