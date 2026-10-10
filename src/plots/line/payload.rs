//! The line renderer's data: a payload of `u32` words, each drawn line's
//! index followed by the bits of its samples (`line.wgsl` reads the buffer as
//! `array<u32>` and bitcasts the samples, so no float load can flush a small
//! index to zero), which payload the buffer holds, and its upload.
//! `OctantApp::upload_line_payload` is the one gate against the device's
//! buffer limit: a layout past it is refused and reported once.

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

/// What a renderer's buffer holds: the key and shape of its payload (`None`
/// before the first upload or after a raw write), and the layout refused for
/// being past the device's limit.
#[derive(Default)]
pub(super) struct PayloadSlot(Mutex<Held>);

#[derive(Clone, Copy, Default)]
struct Held {
    payload: Option<(u64, LineShape)>,
    refused: Option<u64>,
}

impl PayloadSlot {
    fn with<R>(&self, f: impl FnOnce(&mut Held) -> R) -> R {
        f(&mut self.0.lock().unwrap_or_else(|p| p.into_inner()))
    }

    /// The shape of the payload for `key`, when that is what the buffer holds.
    pub(super) fn shape(&self, key: u64) -> Option<LineShape> {
        let held = self.with(|h| h.payload);
        held.filter(|&(k, _)| k == key).map(|(_, shape)| shape)
    }

    /// Records that the buffer holds the payload for `key`.
    pub(super) fn hold(&self, key: u64, shape: LineShape) {
        self.with(|h| h.payload = Some((key, shape)));
    }

    /// Records that the buffer holds no payload.
    pub(super) fn forget(&self) {
        self.with(|h| h.payload = None);
    }

    /// Records layout `layout` as refused; `true` when it was not already.
    fn refuse(&self, layout: u64) -> bool {
        self.with(|h| h.refused.replace(layout) != Some(layout))
    }

    fn refuses(&self, layout: u64) -> bool {
        self.with(|h| h.refused == Some(layout))
    }
}

impl LineRenderer {
    /// The most bytes a line payload may take on `device`: its storage
    /// binding limit, rounded down to whole `u32` words.
    pub fn payload_limit(device: &wgpu::Device) -> u64 {
        let limits = device.limits();
        let limit = limits
            .max_storage_buffer_binding_size
            .min(limits.max_buffer_size);
        limit & !3
    }

    /// The shape of the payload for `key`, when that is what the buffer holds.
    pub fn payload_shape(&self, key: u64) -> Option<LineShape> {
        self.payload.shape(key)
    }

    /// Whether layout `layout` (a hash of the app's `LineLayout`) was refused
    /// for being past the device's limit.
    pub fn refuses(&self, layout: u64) -> bool {
        self.payload.refuses(layout)
    }

    /// Refuses layout `layout`, whose payload takes `needed` bytes, past the
    /// device's `limit`; reports it the first time.
    pub fn refuse(&self, layout: u64, needed: u64, limit: u64) {
        if self.payload.refuse(layout) {
            crate::ui::toast::report(
                crate::ui::toast::Severity::Warning,
                "Line plot too large for the GPU",
                format!("The lines take {needed} bytes; this GPU binds at most {limit}."),
            );
        }
    }

    /// Uploads `words`, a payload of `shape` within the device's limit
    /// (`OctantApp::line_payload`), as the one for `key`.
    pub fn upload_payload(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: u64,
        words: &[u32],
        shape: LineShape,
    ) {
        if !words.is_empty() {
            self.write_data(device, queue, bytemuck::cast_slice(words));
        }
        self.payload.hold(key, shape);
    }

    /// Writes `bytes` (within the device's limit) to the start of the data
    /// buffer, growing it when they do not fit.
    fn write_data(&self, device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) {
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
            return;
        }
        let data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("1D Line Storage Buffer (Resized)"),
            size: buffer_capacity(needed, Self::payload_limit(device)),
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
    }
}

/// Bytes to allocate for `needed` bytes of data: the next power of two, so a
/// growing payload reallocates rarely, but not past the device's `limit`
/// (which `needed` is within).
fn buffer_capacity(needed: u64, limit: u64) -> u64 {
    needed.next_power_of_two().min(limit).max(needed)
}

#[cfg(test)]
mod tests {
    use super::{LineRenderer, LineShape, PayloadSlot, buffer_capacity};

    const SHAPE: LineShape = LineShape {
        profile_length: 2,
        drawn_lines: 1,
        line_count: 3,
    };

    #[test]
    fn capacity_rounds_up_but_stays_under_the_limit() {
        assert_eq!(buffer_capacity(100, 1 << 20), 128);
        assert_eq!(buffer_capacity(129 << 20, 128 << 21), 256 << 20);
        assert_eq!(buffer_capacity(100 << 20, 128 << 20), 128 << 20);
        assert_eq!(buffer_capacity(120 << 20, 125 << 20), 125 << 20);
    }

    #[test]
    fn a_slot_answers_only_for_its_key() {
        let slot = PayloadSlot::default();
        assert_eq!(slot.shape(7), None);
        slot.hold(7, SHAPE);
        assert_eq!((slot.shape(7), slot.shape(8)), (Some(SHAPE), None));
        slot.forget();
        assert_eq!(slot.shape(7), None);
    }

    #[test]
    fn a_layout_is_refused_once() {
        let slot = PayloadSlot::default();
        assert!(!slot.refuses(4));
        assert!(slot.refuse(4), "first refusal reports");
        assert!(!slot.refuse(4), "the same layout again does not");
        assert!(slot.refuses(4));
        assert!(slot.refuse(5) && !slot.refuses(4), "another layout");
    }

    /// The renderer reports the payload it holds only for the key it was
    /// uploaded with, and forgets it after a raw write. Skipped without a GPU.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn line_renderer_tracks_its_uploaded_payload() {
        let Some((device, queue)) = crate::plots::test_gpu::device() else {
            eprintln!("SKIPPED line_renderer_tracks_its_uploaded_payload: no GPU device");
            return;
        };
        let renderer = LineRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm);
        assert_eq!(renderer.payload_shape(7), None, "nothing uploaded yet");
        let payload = [1, 3.0f32.to_bits(), 4.0f32.to_bits()];
        renderer.upload_payload(&device, &queue, 7, &payload, SHAPE);
        assert_eq!(renderer.payload_shape(7), Some(SHAPE));
        assert_eq!(renderer.payload_shape(8), None, "another key");
        renderer.update_data(&queue, &[1.0, 2.0]);
        assert_eq!(renderer.payload_shape(7), None, "overwritten");
        assert_eq!(LineRenderer::payload_limit(&device) % 4, 0, "whole words");
    }
}
