//! Read-ahead for remote files: HDF5 makes many small reads close together (object
//! headers, heaps, B-tree nodes, small chunks), each a round trip if sent alone. Reads
//! smaller than a block are served from aligned `BLOCK`-byte blocks, fetched once and kept
//! (the last `KEPT` of them); larger reads go to the server as they are.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use hdf5_pure::FormatError;

const BLOCK: u64 = 256 << 10;
/// Blocks kept: 16 MiB.
const KEPT: usize = 64;

#[derive(Default)]
struct Kept {
    blocks: HashMap<u64, Arc<[u8]>>,
    order: VecDeque<u64>,
}

#[derive(Default)]
pub struct ReadAhead {
    kept: Mutex<Kept>,
}

impl ReadAhead {
    /// Fills `buf` from `offset` of a file `len` bytes long, `fetch(offset, length)`
    /// reading the server.
    pub fn read(
        &self,
        len: u64,
        offset: u64,
        buf: &mut [u8],
        fetch: impl Fn(u64, u64) -> Result<Vec<u8>, FormatError>,
    ) -> Result<(), FormatError> {
        let end = offset.saturating_add(buf.len() as u64);
        if buf.len() as u64 >= BLOCK {
            return copy_all(&fetch(offset, buf.len() as u64)?, buf);
        }
        let mut at = offset;
        while at < end {
            let index = at / BLOCK;
            let block = self.block(index, len, &fetch)?;
            let start = index * BLOCK;
            let from = (at - start) as usize;
            let to = (end.min(start + block.len() as u64) - start) as usize;
            let Some(part) = block.get(from..to).filter(|p| !p.is_empty()) else {
                return Err(FormatError::Source("Read past the end of the file".into()));
            };
            let done = (at - offset) as usize;
            buf[done..done + part.len()].copy_from_slice(part);
            at += part.len() as u64;
        }
        Ok(())
    }

    fn block(
        &self,
        index: u64,
        len: u64,
        fetch: &impl Fn(u64, u64) -> Result<Vec<u8>, FormatError>,
    ) -> Result<Arc<[u8]>, FormatError> {
        let lock = || self.kept.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(block) = lock().blocks.get(&index) {
            return Ok(block.clone());
        }
        let start = index * BLOCK;
        let block: Arc<[u8]> = fetch(start, BLOCK.min(len.saturating_sub(start)))?.into();
        let mut kept = lock();
        if kept.blocks.insert(index, block.clone()).is_none() {
            kept.order.push_back(index);
        }
        while kept.order.len() > KEPT {
            if let Some(old) = kept.order.pop_front() {
                kept.blocks.remove(&old);
            }
        }
        Ok(block)
    }
}

fn copy_all(bytes: &[u8], buf: &mut [u8]) -> Result<(), FormatError> {
    if bytes.len() != buf.len() {
        return Err(FormatError::Source("Short read".into()));
    }
    buf.copy_from_slice(bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::{BLOCK, ReadAhead};

    #[test]
    fn small_reads_share_blocks_and_large_ones_go_straight() {
        let len = 3 * BLOCK + 10;
        let file: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
        let fetches = Cell::new(0);
        let fetch = |offset: u64, length: u64| {
            fetches.set(fetches.get() + 1);
            Ok(file[offset as usize..(offset + length) as usize].to_vec())
        };
        let ahead = ReadAhead::default();
        let read = |offset: u64, n: usize| {
            let mut buf = vec![0; n];
            ahead.read(len, offset, &mut buf, fetch).expect("read");
            assert_eq!(buf, file[offset as usize..offset as usize + n]);
        };
        read(8, 100);
        read(500, 100);
        assert_eq!(fetches.get(), 1, "one block for both");
        read(BLOCK - 4, 8);
        assert_eq!(fetches.get(), 2, "across a block edge");
        read(3 * BLOCK, 10);
        read(0, BLOCK as usize);
        assert_eq!(fetches.get(), 4, "the short last block, then a large read");
    }
}
