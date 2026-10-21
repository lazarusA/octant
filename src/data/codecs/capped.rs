//! Decompression with a size limit: a small chunk or file must never expand
//! to gigabytes and run the page out of memory.

use std::io::Read;

use zarrs::array::BytesRepresentation;

/// The most one decoded chunk or Icechunk file may hold (the browser's
/// memory ends at 4 GiB).
pub const MAX_DECODED_BYTES: usize = 512 << 20;

/// Reads `reader` to its end, failing once more than `cap` bytes arrive;
/// reserves at most 16 MiB up front.
pub fn read_capped(reader: impl Read, cap: usize) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(cap.min(16 << 20));
    reader
        .take(u64::try_from(cap).unwrap_or(u64::MAX).saturating_add(1))
        .read_to_end(&mut out)?;
    if out.len() > cap {
        return Err(std::io::Error::other(format!(
            "decoded data exceeds {cap} bytes"
        )));
    }
    Ok(out)
}

/// The most a chunk declared as `rep` may decode to: its own size when the
/// codec chain knows it, never more than [`MAX_DECODED_BYTES`].
pub fn cap_for(rep: &BytesRepresentation) -> usize {
    match rep {
        BytesRepresentation::FixedSize(n) | BytesRepresentation::BoundedSize(n) => {
            usize::try_from(*n).map_or(MAX_DECODED_BYTES, |n| n.min(MAX_DECODED_BYTES))
        }
        BytesRepresentation::UnboundedSize => MAX_DECODED_BYTES,
    }
}
