//! Icechunk binary file header validation and payload decompression.

use icechunk_format::format_constants::{
    CompressionAlgorithmBin, ICECHUNK_FILE_HEADER_LEN, ICECHUNK_FORMAT_MAGIC_BYTES, SpecVersionBin,
    parse_file_header,
};

use crate::data::blocks::BlockStoreError;

/// Attempts Zstandard decompression on raw byte slices using `ruzstd`.
pub fn try_decompress_zstd(data: &[u8]) -> Option<Vec<u8>> {
    let decoder = ruzstd::decoding::StreamingDecoder::new(data).ok()?;
    let cap = crate::data::codecs::capped::MAX_DECODED_BYTES;
    crate::data::codecs::capped::read_capped(decoder, cap).ok()
}

/// Decompresses an Icechunk binary file, respecting the 39-byte header and spec version.
pub fn decompress_icechunk_file(data: &[u8]) -> Result<(SpecVersionBin, Vec<u8>), BlockStoreError> {
    if data.is_empty() {
        return Ok((SpecVersionBin::V1, Vec::new()));
    }

    // 1. Check for standard 39-byte Icechunk binary file header
    if data.len() >= ICECHUNK_FILE_HEADER_LEN
        && data.starts_with(ICECHUNK_FORMAT_MAGIC_BYTES)
        && let Ok(header) = parse_file_header(data)
    {
        let payload = &data[ICECHUNK_FILE_HEADER_LEN..];
        let decompressed = match header.compression {
            CompressionAlgorithmBin::Zstd => try_decompress_zstd(payload).ok_or_else(|| {
                BlockStoreError::from("Failed to decompress Zstd payload from Icechunk file")
            })?,
            CompressionAlgorithmBin::None => payload.to_vec(),
        };
        return Ok((header.spec_version, decompressed));
    }

    // 2. Direct Zstandard fallback if raw compressed stream
    if let Some(decompressed) = try_decompress_zstd(data) {
        return Ok((SpecVersionBin::V1, decompressed));
    }

    // 3. Fall back to raw payload if uncompressed
    Ok((SpecVersionBin::V1, data.to_vec()))
}

/// Decompresses raw Icechunk payloads (FlatBuffers or Zstandard).
pub fn decompress_payload(data: &[u8]) -> Result<Vec<u8>, BlockStoreError> {
    decompress_icechunk_file(data).map(|(_, bytes)| bytes)
}
