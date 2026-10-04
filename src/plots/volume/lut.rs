//! Transfer-function lookup texture: 256 `Rgba16Float` texels holding the
//! colormap RGB (atlas row texels, from the registry) and the DVR extinction
//! weight at each scale position. One filtered read per sample replaces the
//! colormap blend; it is rebuilt only when its inputs change.

use crate::plots::common::PlotColorParams;
use crate::utils::colormap::{COLORMAP_RGB_COMPOSITE, LUT_SIZE, registry};
use half::f16;
use std::sync::Mutex;

/// Inputs the lookup texture depends on.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TransferKey {
    row: u32,
    reverse: bool,
    generation: u64,
}

impl TransferKey {
    pub fn new(color: &PlotColorParams) -> Self {
        // Same row choice as WGSL `plot_colormap_row`.
        let row = if color.colormap == COLORMAP_RGB_COMPOSITE {
            color.fallback_colormap
        } else {
            color.colormap
        };
        Self {
            row,
            reverse: color.reverse != 0,
            generation: registry::generation(),
        }
    }
}

/// DVR extinction weight at scale position `t`, in [0, 1]: the shader scales
/// it by Density (optical depth per world unit at the top of the range). The
/// quadratic ramp keeps low values clear, so structure shows through instead of
/// every in-range sample adding fog.
pub fn dvr_extinction(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t
}

/// RGBA texels for `key`, in [0, 1]: texel `i` is the atlas row texel at scale
/// position `i / 255` (reversed if requested) with that position's extinction.
pub fn build_transfer(key: &TransferKey) -> Vec<[f32; 4]> {
    let last = (LUT_SIZE - 1) as f32;
    (0..LUT_SIZE)
        .map(|i| {
            let t = i as f32 / last;
            let along = if key.reverse { 1.0 - t } else { t };
            let c = registry::sample_row(key.row, along, false);
            let [r, g, b] = [c.r(), c.g(), c.b()].map(|v| f32::from(v) / 255.0);
            [r, g, b, dvr_extinction(t)]
        })
        .collect()
}

pub struct TransferLut {
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    key: Mutex<Option<TransferKey>>,
}

impl TransferLut {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Volume Transfer LUT"),
            size: extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
            key: Mutex::new(None),
        }
    }

    /// Uploads the lookup texture when the colormap changed.
    pub fn sync(&self, queue: &wgpu::Queue, color: &PlotColorParams) {
        let key = TransferKey::new(color);
        let mut current = self.key.lock().unwrap_or_else(|p| p.into_inner());
        if *current == Some(key) {
            return;
        }
        let texels: Vec<u16> = build_transfer(&key)
            .into_iter()
            .flatten()
            .map(|v| f16::from_f32(v).to_bits())
            .collect();
        queue.write_texture(
            self.texture.as_image_copy(),
            bytemuck::cast_slice(&texels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(LUT_SIZE as u32 * 8),
                rows_per_image: Some(1),
            },
            extent(),
        );
        *current = Some(key);
    }
}

fn extent() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: LUT_SIZE as u32,
        height: 1,
        depth_or_array_layers: 1,
    }
}
