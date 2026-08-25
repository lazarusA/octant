use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A triple-buffered asynchronous GPU staging ring for non-blocking live video recording.
///
/// Eliminates synchronous GPU-to-CPU stalls (`device.poll(Maintain::Wait)`) by rotating
/// across 3 staging buffers and mapping completed transfers asynchronously with `Maintain::Poll`.
pub struct CaptureRing {
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    unpadded_bytes_per_row: u32,
    buffers: Vec<wgpu::Buffer>,
    slot_mapped_flags: Vec<Arc<AtomicBool>>,
    current_slot: usize,
}

impl CaptureRing {
    /// Creates a new 3-slot staging ring for given dimensions.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let unpadded_bytes_per_row = width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) & !(align - 1);
        let buffer_size = (padded_bytes_per_row * height) as u64;

        let mut buffers = Vec::with_capacity(3);
        let mut slot_mapped_flags = Vec::with_capacity(3);

        for _ in 0..3 {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Octant Capture Ring Staging Buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            buffers.push(buffer);
            slot_mapped_flags.push(Arc::new(AtomicBool::new(false)));
        }

        Self {
            width,
            height,
            padded_bytes_per_row,
            unpadded_bytes_per_row,
            buffers,
            slot_mapped_flags,
            current_slot: 0,
        }
    }

    /// Enqueues a non-blocking texture copy into the current staging buffer slot.
    pub fn enqueue_copy(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source_texture: &wgpu::Texture,
    ) {
        if self.buffers.is_empty() {
            return;
        }

        let slot = self.current_slot % self.buffers.len();
        let target_buffer = &self.buffers[slot];
        let flag = &self.slot_mapped_flags[slot];

        // Ensure buffer is unmapped before copying into it
        if flag.load(Ordering::Acquire) {
            target_buffer.unmap();
            flag.store(false, Ordering::Release);
        }

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: source_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: target_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        // Advance slot
        self.current_slot = (self.current_slot + 1) % self.buffers.len();
    }

    /// Initiates asynchronous mapping on the previous in-flight buffer.
    pub fn request_async_map(&self) {
        if self.buffers.is_empty() {
            return;
        }

        // Map buffer from previous slot (N-1 or N-2)
        let prev_slot = (self.current_slot + self.buffers.len() - 1) % self.buffers.len();
        let buffer = &self.buffers[prev_slot];
        let flag = self.slot_mapped_flags[prev_slot].clone();

        if !flag.load(Ordering::Acquire) {
            let buffer_slice = buffer.slice(..);
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                if result.is_ok() {
                    flag.store(true, Ordering::Release);
                }
            });
        }
    }

    /// Polls completed staging buffers non-blockingly and returns extracted RGBA frames.
    pub fn poll_ready_frames(&self, device: &wgpu::Device) -> Vec<Arc<[u8]>> {
        // Non-blocking poll: immediately returns without stalling CPU thread
        let _ = device.poll(wgpu::PollType::Poll);

        let mut ready_frames = Vec::new();

        for (i, flag) in self.slot_mapped_flags.iter().enumerate() {
            if flag.load(Ordering::Acquire) {
                let buffer = &self.buffers[i];
                let buffer_slice = buffer.slice(..);

                if let Ok(mapped_view) = buffer_slice.get_mapped_range() {
                    let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
                    let padded_row = self.padded_bytes_per_row as usize;
                    let unpadded_row = self.unpadded_bytes_per_row as usize;

                    for row in 0..(self.height as usize) {
                        let start = row * padded_row;
                        let end = start + unpadded_row;
                        if end <= mapped_view.len() {
                            rgba.extend_from_slice(&mapped_view[start..end]);
                        }
                    }

                    drop(mapped_view);
                    ready_frames.push(Arc::from(rgba.into_boxed_slice()));
                }

                buffer.unmap();
                flag.store(false, Ordering::Release);
            }
        }

        ready_frames
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
