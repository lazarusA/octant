//! A bricked volume's GPU side: the atlas
//! of brick slots (`R16Float`, filterable on every device, values stored as
//! `(value - offset) / scale`; `Rgba8Unorm` colors for a composite), the indirection table (`Rgba8Uint`, one
//! texel per level-0 table cell: slot and level + 1) and the uniforms
//! placing the window and the levels (`sample_bricked.wgsl`). Which brick
//! goes where is decided on the CPU (`app::volume_lod`).

use std::ops::Range;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use bytemuck::Pod;

use super::brick_writes::{buffer, texture_3d, write_box};
use super::feedback::Feedback;
use crate::data::multiscale::bricks::{STORED, table_cells};

pub use super::brick_types::{
    AtlasTexel, BrickLevelGpu, BrickUniforms, MAX_LEVELS, SUBS_PER_BRICK,
};

pub struct BrickTextures {
    pub atlas_view: wgpu::TextureView,
    pub table_view: wgpu::TextureView,
    pub uniforms: wgpu::Buffer,
    /// Per slot: min and max valid value, 1 where any is missing.
    pub ranges: wgpu::Buffer,
    /// Per slot, per sub-block (`SUBS_PER_BRICK`): the same.
    pub subs: wgpu::Buffer,
    /// Which table cells the last resting frame's rays reached.
    pub feedback: Feedback,
    slots: [u32; 3],
    /// Bytes per atlas texel: 2 (`R16Float`), 1 (`R8Unorm`) or 4
    /// (`Rgba8Unorm`), and what they hold.
    texel: u32,
    kind: AtlasTexel,
    /// Table cells along x, y, z.
    grid: [u32; 3],
    atlas: wgpu::Texture,
    table: wgpu::Texture,
    /// Counts writes, for the frame cache.
    version: AtomicU64,
    /// Voxels across the window at the finest level drawn.
    dims: [AtomicU32; 3],
}

impl BrickTextures {
    /// An atlas of `slots` per axis holding `texel`s and a table of `grid`
    /// cells, or `None` past the device's 3D texture size.
    pub fn new(
        device: &wgpu::Device,
        slots: [u32; 3],
        grid: [u32; 3],
        kind: AtlasTexel,
    ) -> Option<Self> {
        let max = device.limits().max_texture_dimension_3d;
        let atlas_size = slots.map(|s| s.saturating_mul(STORED as u32));
        if atlas_size.iter().chain(&grid).any(|&n| n == 0 || n > max) {
            log::warn!("Bricked volume skipped: atlas {atlas_size:?} or table {grid:?} past {max}");
            return None;
        }
        let cells = table_cells(grid)?;
        let (format, texel) = kind.format();
        let atlas = texture_3d(device, "Brick Atlas", atlas_size, format);
        let table = texture_3d(device, "Brick Table", grid, wgpu::TextureFormat::Rgba8Uint);
        use wgpu::BufferUsages as U;
        let size = std::mem::size_of::<BrickUniforms>() as u64;
        let uniforms = buffer(device, "Brick Uniforms", size, U::UNIFORM | U::COPY_DST);
        let (ranges, subs) = range_buffers(device, slots)?;
        Some(Self {
            atlas_view: atlas.create_view(&Default::default()),
            table_view: table.create_view(&Default::default()),
            uniforms,
            ranges,
            subs,
            feedback: Feedback::new(device, cells),
            slots,
            texel,
            kind,
            grid,
            atlas,
            table,
            version: AtomicU64::new(0),
            dims: [AtomicU32::new(1), AtomicU32::new(1), AtomicU32::new(1)],
        })
    }

    /// Writes a brick's `STORED`^3 texels (x fastest) into `slot`: f16 bits
    /// (`u16`), bytes (`u8`) or RGBA8 colors (`[u8; 4]`), as the atlas holds.
    pub fn upload_brick<T: Pod>(&self, queue: &wgpu::Queue, slot: [u32; 3], values: &[T]) {
        let n = STORED as u32;
        self.upload_region(queue, slot, [0; 3], [n; 3], values);
    }

    /// Writes `values` (texels as `upload_brick`'s, x fastest) over the box
    /// `at`, `size` of `slot`'s stored cube; nothing when their texel size
    /// is not the atlas's.
    pub fn upload_region<T: Pod>(
        &self,
        queue: &wgpu::Queue,
        slot: [u32; 3],
        at: [u32; 3],
        size: [u32; 3],
        values: &[T],
    ) {
        let texel = std::mem::size_of::<T>() as u32;
        if texel != self.texel {
            log::warn!(
                "Brick of {texel}-byte texels skipped: the atlas holds {}",
                self.texel
            );
            return;
        }
        let n = STORED as u32;
        let origin = std::array::from_fn(|i| slot[i] * n + at[i]);
        write_box(
            queue,
            &self.atlas,
            origin,
            size,
            texel,
            bytemuck::cast_slice(values),
        );
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    /// Writes the table texels `texels` (x fastest) over `cells`.
    pub fn write_table(&self, queue: &wgpu::Queue, cells: &[Range<u32>; 3], texels: &[[u8; 4]]) {
        let origin = cells.clone().map(|r| r.start);
        let size = cells.clone().map(|r| r.end.saturating_sub(r.start));
        write_box(
            queue,
            &self.table,
            origin,
            size,
            4,
            bytemuck::cast_slice(texels),
        );
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    /// Records `slot`'s value range (`min`, `max`) and whether it has
    /// missing voxels, which rays skip it by.
    pub fn write_range(&self, queue: &wgpu::Queue, slot: [u32; 3], range: [f32; 2], missing: bool) {
        let [sx, sy, _] = self.slots;
        let index = u64::from(slot[0] + sx * (slot[1] + sy * slot[2]));
        let texel = [range[0], range[1], f32::from(u8::from(missing)), 0.0];
        queue.write_buffer(&self.ranges, index * 16, bytemuck::bytes_of(&texel));
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    /// Records `slot`'s sub-block ranges (`SUBS_PER_BRICK` of
    /// `[min, max, missing, 0]`, x fastest).
    pub fn write_subs(&self, queue: &wgpu::Queue, slot: [u32; 3], subs: &[[f32; 4]]) {
        let [sx, sy, _] = self.slots;
        let index = u64::from(slot[0] + sx * (slot[1] + sy * slot[2]));
        let offset = index * SUBS_PER_BRICK * 16;
        queue.write_buffer(&self.subs, offset, bytemuck::cast_slice(subs));
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_uniforms(&self, queue: &wgpu::Queue, uniforms: &BrickUniforms) {
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(uniforms));
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    /// Voxels across the window at the finest level drawn (the ray's step).
    pub fn set_dims(&self, dims: [u32; 3]) {
        for (slot, d) in self.dims.iter().zip(dims) {
            slot.store(d.max(1), Ordering::Relaxed);
        }
    }

    pub fn dims(&self) -> [u32; 3] {
        self.dims.each_ref().map(|d| d.load(Ordering::Relaxed))
    }

    /// Whether this atlas and table hold `slots` of `texel`s over `grid`.
    pub fn fits(&self, slots: [u32; 3], grid: [u32; 3], texel: AtlasTexel) -> bool {
        self.slots == slots && self.grid == grid && self.kind == texel
    }

    /// Points every table cell at nothing (bricks are placed anew).
    pub fn clear_table(&self, queue: &wgpu::Queue) {
        let cells = self.grid.map(|g| 0..g);
        let Some(count) = table_cells(self.grid) else {
            return;
        };
        self.write_table(queue, &cells, &vec![[0u8; 4]; count]);
    }

    /// Whether the atlas holds a composite's colors.
    pub fn color(&self) -> bool {
        self.texel == 4
    }

    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }
}

/// The per-slot and per-sub-block range buffers of an atlas of `slots`
/// (16 bytes each), `None` past `u64`.
fn range_buffers(device: &wgpu::Device, slots: [u32; 3]) -> Option<(wgpu::Buffer, wgpu::Buffer)> {
    use wgpu::BufferUsages as U;
    let count = table_cells(slots)?.max(1) as u64;
    let ranges = buffer(
        device,
        "Brick Ranges",
        count.checked_mul(16)?,
        U::STORAGE | U::COPY_DST,
    );
    let subs_size = count.checked_mul(SUBS_PER_BRICK * 16)?;
    let subs = buffer(
        device,
        "Brick Sub-block Ranges",
        subs_size,
        U::STORAGE | U::COPY_DST,
    );
    Some((ranges, subs))
}
