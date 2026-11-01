//! A 3D multi-channel composite blended once into the volume raymarcher's
//! textures (`shaders/channel_bake_volume.wgsl`): its `Rgba8Unorm` colors
//! and `R8Unorm` validity, as `volume/encode.rs` encodes CPU-blended ones,
//! so the raymarch reads what it reads for those and a display change costs
//! one compute pass and a copy.

use wgpu::util::DeviceExt;

use super::channel_bake::Thinning;
use super::channel_params::ChannelParams;
use super::channels::ChannelRenderer;

const SHADER: &str = concat!(
    include_str!("shaders/common/channel_blend.wgsl"),
    "\n",
    include_str!("shaders/channel_bake_volume.wgsl"),
);

/// Validity rows are copied into the texture in rows of this many bytes.
const ROW_ALIGN: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

/// The volume bake's compute pipeline and its group 0 (uniforms, colors,
/// validity words).
pub(super) struct VolumeBakePipeline {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

/// The raymarcher's textures of a `[w, h, d]` volume, what a volume bake
/// writes.
pub struct VolumeTargets<'a> {
    pub id: super::BakeId,
    pub colors: &'a wgpu::Texture,
    pub validity: &'a wgpu::Texture,
    pub dims: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct VolumeBakeUniforms {
    w: u32,
    h: u32,
    d: u32,
    row_words: u32,
}

impl VolumeBakePipeline {
    pub(super) fn new(device: &wgpu::Device, channels: &wgpu::BindGroupLayout) -> Self {
        let compute = wgpu::ShaderStages::COMPUTE;
        let buffer = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: compute,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let colors = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: compute,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_dimension: wgpu::TextureViewDimension::D3,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Channel Volume Bake Layout"),
            entries: &[
                buffer(0, wgpu::BufferBindingType::Uniform),
                colors,
                buffer(2, wgpu::BufferBindingType::Storage { read_only: false }),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Channel Volume Bake"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Channel Volume Bake"),
            bind_group_layouts: &[Some(&layout), None, Some(channels)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Channel Volume Bake"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("bake_volume"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self { pipeline, layout }
    }
}

impl ChannelRenderer {
    /// Blends the planes (one `w * h * d` volume per channel) in `params`
    /// into `targets`, unless they already hold that blend. Returns whether
    /// it baked. `params` must already be written (`update_params`).
    pub(crate) fn bake_volume(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        params: &ChannelParams,
        targets: &VolumeTargets<'_>,
    ) -> bool {
        let key = targets.id;
        let Some(planes_version) = self.bake_needed(key, params, Thinning::NONE) else {
            return false;
        };
        let [w, h, d] = targets.dims.map(|n| n.max(1));
        let row_bytes = w.div_ceil(4) * 4;
        let row_bytes = row_bytes.div_ceil(ROW_ALIGN) * ROW_ALIGN;
        let uniforms = VolumeBakeUniforms {
            w,
            h,
            d,
            row_words: row_bytes / 4,
        };
        let validity_words = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Channel Volume Validity"),
            size: u64::from(row_bytes) * u64::from(h) * u64::from(d),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let bake = self.volume_bake(device);
        let group = bake.targets_group(device, &uniforms, targets.colors, &validity_words);
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&bake.pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.set_bind_group(2, &self.bind_group, &[]);
            pass.dispatch_workgroups(w.div_ceil(4).div_ceil(8), h.div_ceil(8), d);
        }
        copy_validity(
            encoder,
            &validity_words,
            targets.validity,
            row_bytes,
            [w, h, d],
        );
        self.remember_bake(key, params, Thinning::NONE, planes_version);
        true
    }

    /// The volume bake pipeline, built on first use.
    fn volume_bake(&self, device: &wgpu::Device) -> &VolumeBakePipeline {
        self.volume_bake
            .get_or_init(|| VolumeBakePipeline::new(device, &self.layout))
    }
}

impl VolumeBakePipeline {
    /// Group 0 of a bake: its `uniforms`, the `colors` texture it writes and
    /// the `validity` words it fills.
    fn targets_group(
        &self,
        device: &wgpu::Device,
        uniforms: &VolumeBakeUniforms,
        colors: &wgpu::Texture,
        validity: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Channel Volume Bake Uniforms"),
            contents: bytemuck::bytes_of(uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let colors = colors.create_view(&Default::default());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Channel Volume Bake Targets"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&colors),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: validity.as_entire_binding(),
                },
            ],
        })
    }
}

/// Copies the baked validity `words` (`row_bytes` a row) into the `R8Unorm`
/// `validity` texture of `[w, h, d]` voxels, not a WebGPU storage format.
fn copy_validity(
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
