//! Decompression with a size limit: a small chunk or file must never expand
//! to gigabytes and run the page out of memory.

use std::io::Read;

use zarrs::array::BytesRepresentation;

/// The most one decoded chunk or Icechunk file may hold (the browser's
/// memory ends at 4 GiB).
pub const MAX_DECODED_BYTES: usize = 512 << 20;

/// What a read of unknown size reserves up front.
pub const UP_FRONT_BYTES: usize = 16 << 20;

/// Reads `reader` to its end, failing once more than `cap` bytes arrive;
/// reserves `reserve` bytes (at most `cap`) up front.
pub fn read_capped(reader: impl Read, cap: usize, reserve: usize) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(reserve.min(cap));
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

/// What a chunk declared as `rep` reserves up front: all of a fixed size,
/// which it fills exactly (no copies while it grows), else [`UP_FRONT_BYTES`].
pub fn reserve_for(rep: &BytesRepresentation) -> usize {
    match rep {
        BytesRepresentation::FixedSize(_) => cap_for(rep),
        _ => UP_FRONT_BYTES,
    }
}
