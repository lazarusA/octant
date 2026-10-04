//! GPU colormap atlas: a 256×K `Rgba8Unorm` texture with one row per registered
//! colormap, bound at `@group(1)` by every colormapped plot pipeline.
//!
//! The atlas lives in egui-wgpu's `CallbackResources`; each renderer declares an
//! identical bind group layout (WebGPU treats equal layouts as compatible), so a
//! single bind group serves every pipeline.

use crate::utils::colormap::{LUT_SIZE, registry};
use eframe::egui_wgpu::{CallbackResources, RenderState};

/// Rows are allocated in blocks so adding custom maps rarely reallocates.
const ROW_BLOCK: u32 = 64;

pub struct ColormapAtlas {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    rows: u32,
    generation: u64,
}

/// Layout of `@group(1)`: binding 0 is the atlas texture (sampled with `textureLoad`).
pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Colormap Atlas Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }],
    })
}

impl ColormapAtlas {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let rows = row_capacity(device, registry::rows());
        let (texture, bind_group) = create_texture(device, rows);
        let mut atlas = Self {
            texture,
            bind_group,
            rows,
            generation: 0,
        };
        atlas.sync(device, queue);
        atlas
    }

    /// Re-uploads all rows when the registry changed, growing the texture if needed.
    fn sync(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let generation = registry::generation();
        if generation == self.generation {
            return;
        }
        let needed = row_capacity(device, registry::rows());
        if needed > self.rows {
            (self.texture, self.bind_group) = create_texture(device, needed);
            self.rows = needed;
        }
        let row_bytes = LUT_SIZE * 4;
        let mut pixels = vec![0u8; row_bytes * self.rows as usize];
        registry::for_each_lut(|id, lut| {
            let start = id as usize * row_bytes;
            if let Some(dst) = pixels.get_mut(start..start + row_bytes) {
                dst.copy_from_slice(bytemuck::cast_slice(lut.as_slice()));
            }
        });
        queue.write_texture(
            self.texture.as_image_copy(),
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row_bytes as u32),
                rows_per_image: Some(self.rows),
            },
            atlas_extent(self.rows),
        );
        self.generation = generation;
    }
}

fn row_capacity(device: &wgpu::Device, maps: usize) -> u32 {
    let maps = u32::try_from(maps).unwrap_or(u32::MAX);
    let blocks = maps.div_ceil(ROW_BLOCK).max(1);
    blocks
        .saturating_mul(ROW_BLOCK)
        .min(device.limits().max_texture_dimension_2d)
}

fn atlas_extent(rows: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: LUT_SIZE as u32,
        height: rows,
        depth_or_array_layers: 1,
    }
}

fn create_texture(device: &wgpu::Device, rows: u32) -> (wgpu::Texture, wgpu::BindGroup) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Colormap Atlas"),
        size: atlas_extent(rows),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Colormap Atlas Bind Group"),
        layout: &bind_group_layout(device),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&view),
        }],
    });
    (texture, bind_group)
}

/// Creates the atlas, or refreshes it after colormaps were added or removed.
/// Cheap when nothing changed: the registry generation is compared first.
pub fn sync(render_state: &RenderState, last_generation: &mut u64) {
    let generation = registry::generation();
    if *last_generation == generation {
        return;
    }
    let mut renderer = render_state.renderer.write();
    let resources = &mut renderer.callback_resources;
    match resources.get_mut::<ColormapAtlas>() {
        Some(atlas) => atlas.sync(&render_state.device, &render_state.queue),
        None => {
            let atlas = ColormapAtlas::new(&render_state.device, &render_state.queue);
            resources.insert(atlas);
        }
    }
    *last_generation = generation;
}

/// Binds the atlas at `@group(1)`. Returns `false` (skip drawing) when it is missing.
pub fn bind(rpass: &mut wgpu::RenderPass<'static>, resources: &CallbackResources) -> bool {
    let Some(atlas) = resources.get::<ColormapAtlas>() else {
        return false;
    };
    rpass.set_bind_group(1, &atlas.bind_group, &[]);
    true
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "colormap_atlas_tests.rs"]
mod tests;
