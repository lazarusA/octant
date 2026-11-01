//! What a volume bake writes besides the colors texture: its uniforms, the
//! validity words copied into the `R8Unorm` validity texture (not a WebGPU
//! storage format) and the marks of the planes holding data, kept per
//! target (`BakeTargets`).

use wgpu::util::DeviceExt;

use super::channel_bake_volume::VolumeTargets;

/// Validity rows are copied into the texture in rows of this many bytes.
const ROW_ALIGN: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

/// One target's group 0 and the buffers behind it.
pub(super) struct BakeTargets {
    pub(super) id: super::BakeId,
    pub(super) group: wgpu::BindGroup,
    pub(super) validity_words: wgpu::Buffer,
    pub(super) plane_data: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct VolumeBakeUniforms {
    w: u32,
    h: u32,
    d: u32,
    row_words: u32,
}

/// Bytes of one row of `w` validity bytes, padded to whole words and for
/// the copy into the validity texture.
pub(super) fn validity_row_bytes(w: u32) -> u32 {
    (w.max(1).div_ceil(4) * 4).div_ceil(ROW_ALIGN) * ROW_ALIGN
}

/// Bytes of the validity words a bake of a `[w, h, d]` volume fills.
pub fn validity_bytes([w, h, d]: [u32; 3]) -> u64 {
    u64::from(validity_row_bytes(w)) * u64::from(h.max(1)) * u64::from(d.max(1))
}

impl BakeTargets {
    /// Group 0 of bakes into `targets`: their uniforms, the colors texture,
    /// the validity words and the plane marks.
    pub(super) fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        targets: &VolumeTargets<'_>,
    ) -> Self {
        let [w, h, d] = targets.dims.map(|n| n.max(1));
        let uniforms = VolumeBakeUniforms {
            w,
            h,
            d,
            row_words: validity_row_bytes(w) / 4,
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Channel Volume Bake Uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let storage = |label, size| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let validity_words = storage("Channel Volume Validity", validity_bytes([w, h, d]));
        let plane_data = storage("Channel Volume Planes With Data", u64::from(d) * 4);
        let colors = targets.colors.create_view(&Default::default());
        let entry = |binding, resource| wgpu::BindGroupEntry { binding, resource };
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Channel Volume Bake Targets"),
            layout,
            entries: &[
                entry(0, uniform_buffer.as_entire_binding()),
                entry(1, wgpu::BindingResource::TextureView(&colors)),
                entry(2, validity_words.as_entire_binding()),
                entry(3, plane_data.as_entire_binding()),
            ],
        });
        Self {
            id: targets.id,
            group,
            validity_words,
            plane_data,
        }
    }
}

/// Copies the baked validity `words` (`row_bytes` a row) into the `R8Unorm`
/// `validity` texture of `[w, h, d]` voxels, not a WebGPU storage format.
pub(super) fn copy_validity(
    encoder: &mut wgpu::CommandEncoder,
    words: &wgpu::Buffer,
    validity: &wgpu::Texture,
    row_bytes: u32,
    [w, h, d]: [u32; 3],
) {
    encoder.copy_buffer_to_texture(
        wgpu::TexelCopyBufferInfo {
            buffer: words,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row_bytes),
                rows_per_image: Some(h),
            },
        },
        validity.as_image_copy(),
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: d,
        },
    );
}
