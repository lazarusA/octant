//! Volume 3D textures: values (`R32Float`, or `Rgba8Unorm` for RGB composites),
//! validity (`R8Unorm`) and the empty-space brick grid (`Rgba32Float`),
//! uploaded as whole Z planes and brick layers.

use super::bricks;
use super::encode::{self, Dims, Encoded, VolumeEncoding};
use std::ops::Range;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Voxels encoded per `write_texture` call, bounding the CPU staging memory.
const UPLOAD_CHUNK_VOXELS: usize = 1 << 24;

pub struct VolumeTextures {
    pub encoding: VolumeEncoding,
    pub dims: Dims,
    value: wgpu::Texture,
    validity: wgpu::Texture,
    pub value_view: wgpu::TextureView,
    pub validity_view: wgpu::TextureView,
    bricks: wgpu::Texture,
    pub bricks_view: wgpu::TextureView,
    invalid_per_plane: Vec<AtomicU32>,
    invalid_total: AtomicU64,
    /// Counts uploads, so cached frames know the data changed.
    version: AtomicU64,
}

impl VolumeTextures {
    /// Creates empty textures, or `None` (logged) when `dims` exceed the device's
    /// 3D texture limit.
    pub fn new(device: &wgpu::Device, dims: Dims, encoding: VolumeEncoding) -> Option<Self> {
        let max = device.limits().max_texture_dimension_3d as usize;
        if dims.w > max || dims.h > max || dims.d > max {
            log::error!(
                "Volume {}x{}x{} exceeds the GPU 3D texture limit ({max} per axis)",
                dims.w,
                dims.h,
                dims.d
            );
            return None;
        }
        let value_format = match encoding {
            VolumeEncoding::Scalar => wgpu::TextureFormat::R32Float,
            VolumeEncoding::PackedRgb => wgpu::TextureFormat::Rgba8Unorm,
        };
        let value = create_texture(device, "Volume Values", dims, value_format);
        let validity = create_texture(
            device,
            "Volume Validity",
            dims,
            wgpu::TextureFormat::R8Unorm,
        );
        let bricks = create_texture(
            device,
            "Volume Bricks",
            bricks::brick_dims(dims),
            wgpu::TextureFormat::Rgba32Float,
        );
        Some(Self {
            encoding,
            dims,
            bricks_view: bricks.create_view(&wgpu::TextureViewDescriptor::default()),
            bricks,
            value_view: value.create_view(&wgpu::TextureViewDescriptor::default()),
            validity_view: validity.create_view(&wgpu::TextureViewDescriptor::default()),
            value,
            validity,
            // Unwritten planes count as missing until their data arrives.
            invalid_per_plane: (0..dims.d)
                .map(|_| AtomicU32::new(dims.plane() as u32))
                .collect(),
            invalid_total: AtomicU64::new((dims.d * dims.plane()) as u64),
            version: AtomicU64::new(0),
        })
    }

    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    /// Whether any voxel is missing; the shader skips validity reads otherwise.
    pub fn has_invalid(&self) -> bool {
        self.invalid_total.load(Ordering::Relaxed) > 0
    }

    /// Re-encodes and uploads planes `z` of the full volume `values`, plus one
    /// plane on each side, whose missing-voxel fill reads the changed planes.
    pub fn upload_planes(&self, queue: &wgpu::Queue, values: &[f32], z: Range<usize>) {
        let Dims { w, h, d } = self.dims;
        if values.len() != w * h * d {
            log::warn!(
                "Volume upload skipped: {} values for a {w}x{h}x{d} texture",
                values.len()
            );
            return;
        }
        self.version.fetch_add(1, Ordering::Relaxed);
        let start = z.start.saturating_sub(1).min(d);
        let end = z.end.saturating_add(1).min(d);
        let chunk = (UPLOAD_CHUNK_VOXELS / self.dims.plane().max(1)).max(1);
        let mut z0 = start;
        while z0 < end {
            let z1 = (z0 + chunk).min(end);
            match self.encoding {
                VolumeEncoding::Scalar => {
                    self.write(queue, encode::encode_scalar(values, self.dims, z0..z1), 4)
                }
                VolumeEncoding::PackedRgb => {
                    self.write(queue, encode::encode_rgba(values, self.dims, z0..z1), 4)
                }
            }
            z0 = z1;
        }
        self.write_bricks(queue, values, z);
    }

    /// Marks every brick as mattering, turning skipping off (tests compare
    /// against it).
    #[cfg(test)]
    pub fn disable_skipping(&self, queue: &wgpu::Queue) {
        let grid = bricks::brick_dims(self.dims);
        let stats = vec![[f32::MIN, f32::MAX, 1.0, 0.0]; grid.w * grid.h * grid.d];
        let extent = wgpu::Extent3d {
            width: grid.w as u32,
            height: grid.h as u32,
            depth_or_array_layers: grid.d as u32,
        };
        let bytes = bytemuck::cast_slice(&stats);
        write_region(
            queue,
            &self.bricks,
            wgpu::Origin3d::ZERO,
            extent,
            bytes,
            grid.w as u32 * 16,
        );
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    /// Recomputes the brick layers whose footprint covers changed planes `z`.
    fn write_bricks(&self, queue: &wgpu::Queue, values: &[f32], z: Range<usize>) {
        let grid = bricks::brick_dims(self.dims);
        let composite = self.encoding == VolumeEncoding::PackedRgb;
        for layer in bricks::affected_layers(z, self.dims.d) {
            let stats = bricks::brick_stats(values, self.dims, composite, layer..layer + 1);
            let extent = wgpu::Extent3d {
                width: grid.w as u32,
                height: grid.h as u32,
                depth_or_array_layers: 1,
            };
            let origin = wgpu::Origin3d {
                x: 0,
                y: 0,
                z: layer as u32,
            };
            let bytes = bytemuck::cast_slice(&stats);
            write_region(
                queue,
                &self.bricks,
                origin,
                extent,
                bytes,
                grid.w as u32 * 16,
            );
        }
    }

    fn write<T: bytemuck::Pod>(&self, queue: &wgpu::Queue, planes: Encoded<T>, texel_bytes: u32) {
        if planes.z.is_empty() {
            return;
        }
        let (w, h) = (self.dims.w as u32, self.dims.h as u32);
        let extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: planes.z.len() as u32,
        };
        let origin = wgpu::Origin3d {
            x: 0,
            y: 0,
            z: planes.z.start as u32,
        };
        write_region(
            queue,
            &self.value,
            origin,
            extent,
            bytemuck::cast_slice(&planes.texels),
            w * texel_bytes,
        );
        write_region(queue, &self.validity, origin, extent, &planes.validity, w);
        for (zi, invalid) in planes.z.clone().zip(planes.invalid_per_plane) {
            if let Some(slot) = self.invalid_per_plane.get(zi) {
                let old = slot.swap(invalid, Ordering::Relaxed);
                self.invalid_total
                    .fetch_add(u64::from(invalid), Ordering::Relaxed);
                self.invalid_total
                    .fetch_sub(u64::from(old), Ordering::Relaxed);
            }
        }
    }
}

fn write_region(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    origin: wgpu::Origin3d,
    extent: wgpu::Extent3d,
    bytes: &[u8],
    bytes_per_row: u32,
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(extent.height),
        },
        extent,
    );
}

fn create_texture(
    device: &wgpu::Device,
    label: &str,
    dims: Dims,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: dims.w as u32,
            height: dims.h as u32,
            depth_or_array_layers: dims.d as u32,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}
