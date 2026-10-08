//! Volume 3D textures: values (`R32Float`, or `Rgba8Unorm` for RGB composites),
//! and validity (`R8Unorm`), uploaded as whole Z planes.

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
            crate::ui::toast::report(
                crate::ui::toast::Severity::Error,
                "Volume too large for the GPU",
                &format!(
                    "Volume {}x{}x{} exceeds GPU 3D texture limit ({max})",
                    dims.w, dims.h, dims.d
                ),
            );
            return None;
        }
        let format = match encoding {
            VolumeEncoding::Scalar => wgpu::TextureFormat::R32Float,
            VolumeEncoding::PackedRgb => wgpu::TextureFormat::Rgba8Unorm,
        };
        let value = create_texture(device, "Volume Values", dims, format);
        let validity = create_texture(
            device,
            "Volume Validity",
            dims,
            wgpu::TextureFormat::R8Unorm,
        );
        let plane = dims.plane();
        Some(Self {
            encoding,
            dims,
            value_view: value.create_view(&wgpu::TextureViewDescriptor::default()),
            validity_view: validity.create_view(&wgpu::TextureViewDescriptor::default()),
            value,
            validity,
            invalid_per_plane: (0..dims.d).map(|_| AtomicU32::new(plane as u32)).collect(),
            invalid_total: AtomicU64::new((dims.d * plane) as u64),
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

    /// Uploads planes `z` of the full volume `values`, plus one plane on each
    /// side (their missing-voxel fill reads the changed planes).
    pub fn upload_planes(&self, queue: &wgpu::Queue, values: &[f32], z: Range<usize>) {
        let Dims { w, h, d } = self.dims;
        if values.len() != w * h * d {
            crate::ui::toast::report(
                crate::ui::toast::Severity::Warning,
                "Volume upload skipped",
                &format!("{} values for a {w}x{h}x{d} texture", values.len()),
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
                VolumeEncoding::Scalar => self.upload_scalar(queue, values, z0..z1),
                VolumeEncoding::PackedRgb => {
                    self.write(queue, encode::encode_rgba(values, self.dims, z0..z1), 4)
                }
            }
            z0 = z1;
        }
    }

    /// Scalar planes `z`: without missing voxels (the common case) the values
    /// upload as they are, and validity only where planes had missing voxels.
    fn upload_scalar(&self, queue: &wgpu::Queue, values: &[f32], z: Range<usize>) {
        let plane = self.dims.plane();
        let slice = &values[z.start * plane..z.end * plane];
        if !encode::all_valid(slice) {
            return self.write(queue, encode::encode_scalar(values, self.dims, z), 4);
        }
        let (origin, extent) = self.region(&z);
        let w = self.dims.w as u32;
        write_region(
            queue,
            &self.value,
            origin,
            extent,
            bytemuck::cast_slice(slice),
            w * 4,
        );
        let stale = (z.start..z.end).any(|zi| {
            self.invalid_per_plane
                .get(zi)
                .is_some_and(|c| c.load(Ordering::Relaxed) != 0)
        });
        if stale {
            write_valid_planes(queue, &self.validity, self.dims, z.clone());
            self.set_invalid_counts(z.start, std::iter::repeat_n(0, z.len()));
        }
    }

    fn region(&self, z: &Range<usize>) -> (wgpu::Origin3d, wgpu::Extent3d) {
        region_3d(self.dims, z.clone())
    }

    fn set_invalid_counts(&self, z0: usize, counts: impl Iterator<Item = u32>) {
        for (zi, invalid) in (z0..).zip(counts) {
            let Some(slot) = self.invalid_per_plane.get(zi) else {
                continue;
            };
            let old = slot.swap(invalid, Ordering::Relaxed);
            if invalid > old {
                self.invalid_total
                    .fetch_add(u64::from(invalid - old), Ordering::Relaxed);
            } else if old > invalid {
                self.invalid_total
                    .fetch_sub(u64::from(old - invalid), Ordering::Relaxed);
            }
        }
    }

    fn write<T: bytemuck::Pod>(&self, queue: &wgpu::Queue, planes: Encoded<T>, texel_bytes: u32) {
        if planes.z.is_empty() {
            return;
        }
        let w = self.dims.w as u32;
        let (origin, extent) = self.region(&planes.z);
        write_region(
            queue,
            &self.value,
            origin,
            extent,
            bytemuck::cast_slice(&planes.texels),
            w * texel_bytes,
        );
        write_region(queue, &self.validity, origin, extent, &planes.validity, w);
        self.set_invalid_counts(planes.z.start, planes.invalid_per_plane.into_iter());
    }
}

fn region_3d(dims: Dims, z: Range<usize>) -> (wgpu::Origin3d, wgpu::Extent3d) {
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

fn write_region(
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

fn write_valid_planes(queue: &wgpu::Queue, texture: &wgpu::Texture, dims: Dims, z: Range<usize>) {
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

fn create_texture(
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
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}
