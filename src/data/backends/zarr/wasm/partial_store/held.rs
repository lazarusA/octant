//! What a [`PartialStore`](super::PartialStore) holds besides whole
//! objects: parts of objects, the objects the budget may drop, and leases.

use std::collections::HashMap;

use zarrs::storage::byte_range::ByteRange;
use zarrs::storage::{Bytes, StoreKey};

/// Parts of an object of `size` bytes: `(offset, bytes)`, in no order.
#[derive(Default)]
pub(super) struct Parts {
    pub(super) size: u64,
    pub(super) pieces: Vec<(u64, Bytes)>,
}

impl Parts {
    /// The bytes of `range` (of `self.size`) when one piece holds them
    /// (zarrs reads each inner chunk whole, within one fetched range).
    pub(super) fn read(&self, range: ByteRange) -> Option<Bytes> {
        let want = range.to_range(self.size);
        self.pieces.iter().find_map(|(offset, bytes)| {
            let end = offset + bytes.len() as u64;
            (want.start >= *offset && want.end <= end).then(|| {
                let from = (want.start - offset) as usize;
                bytes.slice(from..from + (want.end - want.start) as usize)
            })
        })
    }

    /// Whether `start..end` lies within one held piece.
    pub(super) fn holds(&self, start: u64, end: u64) -> bool {
        self.pieces
            .iter()
            .any(|(o, b)| start >= *o && end <= o + b.len() as u64)
    }
}

/// An object the budget may drop: the bytes it holds, and its last use.
pub(super) struct Use {
    pub(super) bytes: u64,
    pub(super) last_used: u64,
}

#[derive(Default)]
pub(super) struct Held {
    pub(super) parts: HashMap<StoreKey, Parts>,
    pub(super) uses: HashMap<StoreKey, Use>,
    /// Leases on keys (counted) and on key prefixes (one entry per lease).
    pub(super) pins: HashMap<StoreKey, u32>,
    pub(super) prefix_pins: Vec<String>,
}

impl Held {
    pub(super) fn pinned(&self, key: &StoreKey) -> bool {
        self.pins.contains_key(key)
            || self
                .prefix_pins
                .iter()
                .any(|p| key.as_str().starts_with(p.as_str()))
    }

    /// Drops the parts of `key` and stops counting it; the bytes counted.
    pub(super) fn forget(&mut self, key: &StoreKey) -> u64 {
        self.parts.remove(key);
        self.uses.remove(key).map_or(0, |u| u.bytes)
    }
}
