//! Decompression with a size limit: a small chunk or file must never expand
//! to gigabytes and run the page out of memory.

use std::io::{Read, Write};

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
        return Err(exceeded(cap));
    }
    Ok(out)
}

/// The error of a decoder whose output passed `cap` bytes.
pub fn exceeded(cap: usize) -> std::io::Error {
    std::io::Error::other(format!("decoded data exceeds {cap} bytes"))
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

/// A writer for decoders that push their output (LZW), failing once more
/// than `cap` bytes arrive.
pub struct CappedWriter {
    out: Vec<u8>,
    cap: usize,
}

impl CappedWriter {
    pub fn new(cap: usize, reserve: usize) -> Self {
        Self {
            out: Vec::with_capacity(reserve.min(cap)),
            cap,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.out
    }
}

impl Write for CappedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.out.len().saturating_add(buf.len()) > self.cap {
            return Err(exceeded(self.cap));
        }
        self.out.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
