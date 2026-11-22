//! Ray feedback (ray-guided streaming, Crassin et al. 2009, Hadwiger et
//! al. 2012): the
//! bricked raymarch sets one bit per level-0 table cell a ray enters before
//! it stops (opaque, or a MIP past its color range), `brick_seen` in
//! `sample_bricked.wgsl`. A finished resting frame copies the bits to a
//! staging buffer, read back without blocking on a later frame
//! (`poll`, `take`), tagged with the frame's parameters (camera, mode,
//! colors, opacity, cut) so the app uses only the bits of the frame it
//! rests on (`app::volume_lod`).

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use super::VolumeUniformParams;
use crate::data::multiscale::bricks::CellBits;

/// Where the readback is.
const IDLE: u8 = 0;
const COPIED: u8 = 1;
const MAPPING: u8 = 2;
const MAPPED: u8 = 3;

pub struct Feedback {
    /// One bit per table cell (x fastest), cleared before each frame.
    pub seen: wgpu::Buffer,
    staging: wgpu::Buffer,
    cells: usize,
    stage: Arc<AtomicU8>,
    /// The parameters of the frame copied, and the bits read back for it.
    copied: Mutex<Option<VolumeUniformParams>>,
    /// A finished frame whose copy waited for the last readback; its bits
    /// stay in `seen` until the next frame clears them.
    missed: Mutex<Option<VolumeUniformParams>>,
    ready: Mutex<Option<(VolumeUniformParams, CellBits)>>,
}

impl Feedback {
    /// Bits for `cells` table cells.
    pub fn new(device: &wgpu::Device, cells: usize) -> Self {
        let size = (cells.div_ceil(32).max(1) * 4) as u64;
        let buffer = |label, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        use wgpu::BufferUsages as U;
        Self {
            seen: buffer("Brick Seen", U::STORAGE | U::COPY_SRC | U::COPY_DST),
            staging: buffer("Brick Seen Readback", U::MAP_READ | U::COPY_DST),
            cells,
            stage: Arc::new(AtomicU8::new(IDLE)),
            copied: Mutex::new(None),
            missed: Mutex::new(None),
            ready: Mutex::new(None),
        }
    }

    /// Clears the bits before a frame's first rows.
    pub fn clear(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.clear_buffer(&self.seen, 0, None);
        *self.missed.lock().unwrap_or_else(|p| p.into_inner()) = None;
    }

    /// Copies the bits of a finished frame drawn with `frame` for readback,
    /// unless the last copy is still being read.
    /// A copy the last readback held back is made by the next `poll`.
    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, frame: VolumeUniformParams) {
        if self.stage.load(Ordering::Acquire) != IDLE {
            *self.missed.lock().unwrap_or_else(|p| p.into_inner()) = Some(frame);
            return;
        }
        encoder.copy_buffer_to_buffer(&self.seen, 0, &self.staging, 0, self.staging.size());
        *self.copied.lock().unwrap_or_else(|p| p.into_inner()) = Some(frame);
        self.stage.store(COPIED, Ordering::Release);
    }

    /// Moves the readback along: maps a copy submitted since the last call,
    /// reads a mapped one. Call once per frame, after the frame that
    /// recorded the copy was submitted; `device.poll` runs the callback.
    pub fn poll(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.stage.load(Ordering::Acquire) == IDLE {
            let missed = self.missed.lock().unwrap_or_else(|p| p.into_inner()).take();
            if let Some(frame) = missed {
                let mut encoder = device.create_command_encoder(&Default::default());
                self.record(&mut encoder, frame);
                queue.submit([encoder.finish()]);
            }
        }
        match self.stage.load(Ordering::Acquire) {
            COPIED => {
                self.stage.store(MAPPING, Ordering::Release);
                let stage = self.stage.clone();
                self.staging
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |r| {
                        let next = if r.is_ok() { MAPPED } else { IDLE };
                        stage.store(next, Ordering::Release);
                    });
            }
            MAPPED => self.read(),
            _ => {}
        }
        let _ = device.poll(wgpu::PollType::Poll);
    }

    /// Whether a readback is under way or a held-back copy waits for the
    /// next `poll` (the app keeps repainting for it).
    pub fn waiting(&self) -> bool {
        self.stage.load(Ordering::Acquire) != IDLE
            || self
                .missed
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .is_some()
    }

    /// Drops what the last plot left: a held-back copy, the tag of one being
    /// read (its bits are then dropped, `read`) and bits not yet taken.
    pub fn forget(&self) {
        *self.missed.lock().unwrap_or_else(|p| p.into_inner()) = None;
        *self.copied.lock().unwrap_or_else(|p| p.into_inner()) = None;
        *self.ready.lock().unwrap_or_else(|p| p.into_inner()) = None;
    }

    /// The bits of the last frame read back, and its parameters.
    pub fn take(&self) -> Option<(VolumeUniformParams, CellBits)> {
        self.ready.lock().unwrap_or_else(|p| p.into_inner()).take()
    }

    fn read(&self) {
        let words: Vec<u32> = match self.staging.slice(..).get_mapped_range() {
            Ok(data) => bytemuck::cast_slice(&data).to_vec(),
            Err(e) => {
                // Dropped: no bits would hide every brick.
                log::debug!("Ray feedback not read: {e}");
                self.staging.unmap();
                self.stage.store(IDLE, Ordering::Release);
                return;
            }
        };
        self.staging.unmap();
        let bits = CellBits::from_words(words, self.cells);
        let frame = self.copied.lock().unwrap_or_else(|p| p.into_inner()).take();
        if let Some(frame) = frame {
            *self.ready.lock().unwrap_or_else(|p| p.into_inner()) = Some((frame, bits));
        }
        self.stage.store(IDLE, Ordering::Release);
    }
}
