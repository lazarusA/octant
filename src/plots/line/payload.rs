//! The line renderer's data: a payload of `u32` words, each drawn line's
//! index followed by the bits of its samples (`line.wgsl` reads the buffer as
//! `array<u32>` and bitcasts the samples, so no float load can flush a small
//! index to zero), which payload the buffer holds, and its upload.

use std::sync::Mutex;

use super::LineRenderer;

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

impl LineRenderer {
    /// The most bytes a line payload may take on `device`: its storage
    /// binding limit.
    pub fn payload_limit(device: &wgpu::Device) -> u64 {
        let limits = device.limits();
        limits
            .max_storage_buffer_binding_size
            .min(limits.max_buffer_size)
    }

    /// The shape of the payload for `key`, when that is what the buffer holds.
    pub fn payload_shape(&self, key: u64) -> Option<LineShape> {
        self.payload.shape(key)
    }

    /// Uploads `words`, a payload of `shape` (`OctantApp::line_payload`), as
    /// the one for `key`. A payload past the device's limit draws nothing.
    pub fn upload_payload(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: u64,
        words: &[u32],
        shape: LineShape,
    ) {
        let uploaded =
            words.is_empty() || self.write_data(device, queue, bytemuck::cast_slice(words));
        let drawn_lines = if uploaded { shape.drawn_lines } else { 0 };
        // Held either way, so a payload that does not fit is reported once.
        self.payload.hold(
            key,
            LineShape {
                drawn_lines,
                ..shape
            },
        );
    }

    /// Records a payload of `needed` bytes, past the device's `limit`, as
    /// drawing nothing for `key`, and reports it once.
    pub fn skip_payload(&self, key: u64, shape: LineShape, needed: u64, limit: u64) {
        report_too_large(needed, limit);
        self.payload.hold(
            key,
            LineShape {
                drawn_lines: 0,
                ..shape
            },
        );
    }

    /// Writes `bytes` to the start of the data buffer, growing it (to the next
    /// power of two, within the device limit) when they do not fit. Reports
    /// and returns `false` when they exceed the limit.
    fn write_data(&self, device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) -> bool {
        let needed = bytes.len() as u64;
        let current = self
            .gpu_resources
            .read()
            .map(|g| g.data_buffer.size())
            .unwrap_or(0);
        if needed <= current {
            if let Ok(guard) = self.gpu_resources.read() {
                queue.write_buffer(&guard.data_buffer, 0, bytes);
            }
            return true;
        }
        let limit = Self::payload_limit(device);
        let Some(capacity) = buffer_capacity(needed, limit) else {
            report_too_large(needed, limit);
            return false;
        };
        let data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("1D Line Storage Buffer (Resized)"),
            size: capacity,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = crate::plots::common::create_uniform_storage_bind_group(
            device,
            "1D Line Bind Group (Resized)",
            &self.bind_group_layout,
            &self.uniform_buffer,
            &data_buffer,
        );
        queue.write_buffer(&data_buffer, 0, bytes);
        if let Ok(mut guard) = self.gpu_resources.write() {
            guard.data_buffer = data_buffer;
            guard.bind_group = bind_group;
        }
        true
    }
}

/// Reports a line plot of `needed` bytes past the device's `limit`.
fn report_too_large(needed: u64, limit: u64) {
    crate::ui::toast::report(
        crate::ui::toast::Severity::Warning,
        "Line plot too large for the GPU",
        format!("The lines take {needed} bytes; this GPU binds at most {limit}."),
    );
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
