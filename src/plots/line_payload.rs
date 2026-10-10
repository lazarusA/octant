//! The line renderer's data: a payload of `u32` words, each drawn line's
//! index followed by the bits of its samples (`line.wgsl` reads the buffer as
//! `array<u32>` and bitcasts the samples, so no float load can flush a small
//! index to zero), and which payload the buffer holds.

use std::sync::Mutex;

/// What a line payload holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineShape {
    pub profile_length: u32,
    /// Lines in the payload.
    pub drawn_lines: u32,
    /// Lines in the plot, drawn or not (the series the colors spread over).
    pub line_count: u32,
}

/// The key and shape of the payload in a renderer's buffer; `None` before
/// the first upload or after a raw write overwrote it.
#[derive(Default)]
pub(super) struct PayloadSlot(Mutex<Option<(u64, LineShape)>>);

impl PayloadSlot {
    /// The shape of the payload for `key`, when that is what the buffer holds.
    pub(super) fn shape(&self, key: u64) -> Option<LineShape> {
        let held = *self.0.lock().unwrap_or_else(|p| p.into_inner());
        held.filter(|&(k, _)| k == key).map(|(_, shape)| shape)
    }

    /// Records that the buffer holds the payload for `key`.
    pub(super) fn hold(&self, key: u64, shape: LineShape) {
        *self.0.lock().unwrap_or_else(|p| p.into_inner()) = Some((key, shape));
    }

    /// Records that the buffer holds no payload.
    pub(super) fn forget(&self) {
        *self.0.lock().unwrap_or_else(|p| p.into_inner()) = None;
    }
}

/// Bytes to allocate for `needed` bytes of data: the next power of two, so a
/// growing payload reallocates rarely, but never past the device's `limit`;
/// `None` when `needed` itself is past it.
pub(super) fn buffer_capacity(needed: u64, limit: u64) -> Option<u64> {
    (needed <= limit).then(|| needed.next_power_of_two().min(limit))
}

#[cfg(test)]
mod tests {
    use super::{LineShape, PayloadSlot, buffer_capacity};

    #[test]
    fn capacity_rounds_up_but_stays_under_the_limit() {
        assert_eq!(buffer_capacity(100, 1 << 20), Some(128));
        assert_eq!(buffer_capacity(129 << 20, 128 << 21), Some(256 << 20));
        assert_eq!(
            buffer_capacity(129 << 20, 128 << 20),
            None,
            "past the limit"
        );
        assert_eq!(buffer_capacity(100 << 20, 128 << 20), Some(128 << 20));
        assert_eq!(buffer_capacity(120 << 20, 125 << 20), Some(125 << 20));
    }

    #[test]
    fn a_slot_answers_only_for_its_key() {
        let slot = PayloadSlot::default();
        let shape = LineShape {
            profile_length: 2,
            drawn_lines: 1,
            line_count: 3,
        };
        assert_eq!(slot.shape(7), None);
        slot.hold(7, shape);
        assert_eq!((slot.shape(7), slot.shape(8)), (Some(shape), None));
        slot.forget();
        assert_eq!(slot.shape(7), None);
    }
}
