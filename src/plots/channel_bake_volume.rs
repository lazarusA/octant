//! A 3D multi-channel composite blended once into the volume raymarcher's
//! textures (`shaders/channel_bake_volume.wgsl`): its `Rgba8Unorm` colors
//! and `R8Unorm` validity, as `volume/encode.rs` encodes CPU-blended ones,
//! so the raymarch reads what it reads for those and a display change costs
//! two compute passes (marking the planes holding data, then the bake) and
//! a copy.

use std::sync::Mutex;

use super::channel_bake::Thinning;
use super::channel_bake_targets::{BakeTargets, copy_validity, validity_bytes, validity_row_bytes};
use super::channel_params::ChannelParams;
use super::channels::ChannelRenderer;

pub(super) const SHADER: &str = concat!(
    include_str!("shaders/common/channel_blend.wgsl"),
    "\n",
    include_str!("shaders/channel_bake_volume.wgsl"),
);

/// The volume bake's compute pipelines (marking the planes holding data,
/// then the bake) and their group 0 (uniforms, colors, validity words,
/// plane marks), with the last target's buffers.
pub(super) struct VolumeBakePipeline {
    mark: wgpu::ComputePipeline,
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    /// Buffers kept for the next bake of the same target (each display
    /// change and ring step bakes again), replaced by another target's.
    targets: Mutex<Option<BakeTargets>>,
}

/// The raymarcher's textures of a `[w, h, d]` volume, what a volume bake
/// writes.
pub struct VolumeTargets<'a> {
    pub id: super::BakeId,
    pub colors: &'a wgpu::Texture,
    pub validity: &'a wgpu::Texture,
    pub dims: [u32; 3],
}

impl VolumeBakePipeline {
    pub(super) fn new(device: &wgpu::Device, channels: &wgpu::BindGroupLayout) -> Self {
        let layout = targets_layout(device);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Channel Volume Bake"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Channel Volume Bake"),
            bind_group_layouts: &[Some(&layout), None, Some(channels)],
            immediate_size: 0,
        });
        let entry = |name| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some(name),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        Self {
            mark: entry("mark_planes"),
            pipeline: entry("bake_volume"),
            layout,
            targets: Mutex::new(None),
        }
    }
}

impl ChannelRenderer {
    /// Blends the planes (one `w * h * d` volume per channel) in `params`
    /// into `targets`, unless they already hold that blend or its validity
    /// words pass one storage binding (planes kept for a point cloud, drawn
    /// as a volume before its reprojection). Returns whether it baked.
    /// `params` must already be written (`update_params`).
    pub(crate) fn bake_volume(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        params: &ChannelParams,
        targets: &VolumeTargets<'_>,
    ) -> bool {
        let limit = device.limits().max_storage_buffer_binding_size;
        if validity_bytes(targets.dims) > limit {
            return false;
        }
        let key = targets.id;
        let Some(planes_version) = self.bake_needed(key, params, Thinning::NONE) else {
            return false;
        };
        self.volume_bake(device)
            .encode(device, encoder, &self.bind_group, targets);
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
    /// Marks the planes of `targets` holding data, bakes them (with the
    /// channels' group 2) and copies the validity words into its texture.
    fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        channels: &wgpu::BindGroup,
        targets: &VolumeTargets<'_>,
    ) {
        let [w, h, d] = targets.dims.map(|n| n.max(1));
        let mut kept = self.targets.lock().unwrap_or_else(|p| p.into_inner());
        let t = match kept.take() {
            Some(t) if t.id == targets.id => t,
            _ => BakeTargets::new(device, targets),
        };
        let group = t.group(device, &self.layout, targets.colors);
        encoder.clear_buffer(&t.plane_data, 0, None);
        for pipeline in [&self.mark, &self.pipeline] {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.set_bind_group(2, channels, &[]);
            pass.dispatch_workgroups(w.div_ceil(4).div_ceil(8), h.div_ceil(8), d);
        }
        let row_bytes = validity_row_bytes(w);
        copy_validity(
            encoder,
            &t.validity_words,
            targets.validity,
            row_bytes,
            [w, h, d],
        );
        *kept = Some(t);
    }
}

/// Group 0 of the volume bake: uniforms, the colors it writes, the
/// validity words and the plane marks.
fn targets_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Channel Volume Bake Layout"),
        entries: &[
            buffer(0, wgpu::BufferBindingType::Uniform),
            colors,
            buffer(2, wgpu::BufferBindingType::Storage { read_only: false }),
            buffer(3, wgpu::BufferBindingType::Storage { read_only: false }),
        ],
    })
}
