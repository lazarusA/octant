//! Creating the volume's 3D textures and writing whole Z planes into them.

use super::encode::Dims;
use std::ops::Range;

pub(super) fn region_3d(dims: Dims, z: Range<usize>) -> (wgpu::Origin3d, wgpu::Extent3d) {
    (
        wgpu::Origin3d {
            x: 0,
            y: 0,
            z: z.start as u32,
        },
        wgpu::Extent3d {
            width: dims.w as u32,
            height: dims.h as u32,
            depth_or_array_layers: z.len() as u32,
        },
    )
}

pub(super) fn write_region(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    origin: wgpu::Origin3d,
    extent: wgpu::Extent3d,
    bytes: &[u8],
    bytes_per_row: u32,
) {
    let layout = wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(bytes_per_row),
        rows_per_image: Some(extent.height),
    };
    let tex = wgpu::TexelCopyTextureInfo {
        texture,
        mip_level: 0,
        origin,
        aspect: wgpu::TextureAspect::All,
    };
    queue.write_texture(tex, bytes, layout, extent);
}

pub(super) fn write_valid_planes(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    dims: Dims,
    z: Range<usize>,
) {
    const CHUNK: usize = 65536;
    static ONES: [u8; CHUNK] = [255u8; CHUNK];
    let plane = dims.plane();
    let step = (CHUNK / plane.max(1)).max(1);
    let mut z0 = z.start;
    while z0 < z.end {
        let z1 = (z0 + step).min(z.end);
        let voxels = (z1 - z0) * plane;
        let buf = if voxels <= CHUNK {
            &ONES[..voxels]
        } else {
            &vec![255; voxels]
        };
        let (origin, extent) = region_3d(dims, z0..z1);
        write_region(queue, texture, origin, extent, buf, dims.w as u32);
        z0 = z1;
    }
}

pub(super) fn create_texture(
    device: &wgpu::Device,
    label: &str,
    dims: Dims,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width: dims.w as u32,
        height: dims.h as u32,
        depth_or_array_layers: dims.d as u32,
    };
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format,
        // Composite colors may be baked on the GPU (`channel_bake_volume`).
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | match format {
                wgpu::TextureFormat::Rgba8Unorm => wgpu::TextureUsages::STORAGE_BINDING,
                _ => wgpu::TextureUsages::empty(),
            },
        view_formats: &[],
    })
}
