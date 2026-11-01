//! A multi-channel composite blended once into a plot's own data buffer:
//! the surface, sphere and point cloud draw packed colors as they draw a
//! CPU-blended composite, so a frame costs what it costs on the CPU path,
//! while a display change or new planes cost one compute pass
//! (`shaders/channel_bake.wgsl`), run only when they change.

use std::sync::atomic::{AtomicU64, Ordering};

use wgpu::util::DeviceExt;

use super::channel_params::ChannelParams;
use super::channels::ChannelRenderer;

const SHADER: &str = concat!(
    include_str!("shaders/common/channel_blend.wgsl"),
    "\n",
    include_str!("shaders/channel_bake.wgsl"),
);

const WORKGROUP: u32 = 64;

/// The bake's compute pipeline and its group 0 (uniforms, target buffer).
#[derive(Clone)]
pub(super) struct BakePipeline {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

/// What a target was last baked from.
pub(super) struct Baked {
    params: ChannelParams,
    planes_version: u64,
    thinning: Thinning,
    target: BakeId,
}

/// What a bake writes, a plot's data buffer or the raymarcher's color
/// texture (`channel_bake_volume.rs`), named by an id never reused, so the
/// memo holds no GPU resource and takes no new target for an old one.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BakeId(u64);

impl BakeId {
    /// A new target's id.
    pub fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// Targets whose last bake is remembered (a mesh and its 3D coastlines).
const REMEMBERED: usize = 4;

/// A target holding a thinned grid of the planes' 2D cells
/// (`MatrixData::decimated`): one cell in `stride` of `full_width` x
/// `full_height`, `width` cells a row. `NONE` when its cells are the planes'.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Thinning {
    pub stride: u32,
    pub width: u32,
    pub full_width: u32,
    pub full_height: u32,
}

impl Thinning {
    pub const NONE: Self = Self {
        stride: 1,
        width: 1,
        full_width: 1,
        full_height: 1,
    };
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BakeUniforms {
    cells: u32,
    row: u32,
    stride: u32,
    width: u32,
    full_width: u32,
    full_height: u32,
    _pad: [u32; 2],
}

impl BakePipeline {
    pub(super) fn new(device: &wgpu::Device, channels: &wgpu::BindGroupLayout) -> Self {
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Channel Bake Layout"),
            entries: &[
                entry(0, wgpu::BufferBindingType::Uniform),
                entry(1, wgpu::BufferBindingType::Storage { read_only: false }),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Channel Bake"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Channel Bake"),
            bind_group_layouts: &[Some(&layout), None, Some(channels)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Channel Bake"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("bake_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self { pipeline, layout }
    }
}

impl ChannelRenderer {
    /// Blends the planes in `params` into `target`, named `id` (one `f32` per cell,
    /// at most a plane's cells, thinned by `thinning`), unless it already
    /// holds that blend of these planes. Returns whether it baked. `params`
    /// must already be written (`update_params`).
    pub(crate) fn bake(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        params: &ChannelParams,
        (key, target, thinning): (BakeId, &wgpu::Buffer, Thinning),
    ) -> bool {
        let Some(planes_version) = self.bake_needed(key, params, thinning) else {
            return false;
        };
        let cells = (target.size() / 4).min(self.plane_len as u64) as u32;
        let groups = cells.div_ceil(WORKGROUP).max(1);
        let max = device.limits().max_compute_workgroups_per_dimension.max(1);
        let (x, y) = (groups.min(max), groups.div_ceil(max));
        let uniforms = BakeUniforms {
            cells,
            row: x * WORKGROUP,
            stride: thinning.stride,
            width: thinning.width.max(1),
            full_width: thinning.full_width.max(1),
            full_height: thinning.full_height.max(1),
            _pad: [0; 2],
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Channel Bake Uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bake = &self.bake;
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Channel Bake Target"),
            layout: &bake.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: target.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&bake.pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.set_bind_group(2, &self.bind_group, &[]);
        pass.dispatch_workgroups(x, y, 1);
        self.remember_bake(key, params, thinning, planes_version);
        true
    }

    /// The planes' version when `target` does not hold their blend in
    /// `params` and `thinning` yet; `None` when it does.
    pub(super) fn bake_needed(
        &self,
        target: BakeId,
        params: &ChannelParams,
        thinning: Thinning,
    ) -> Option<u64> {
        let planes_version = self.planes_version.load(Ordering::Acquire);
        let baked = self.baked.lock().unwrap_or_else(|p| p.into_inner());
        let done = baked.iter().any(|b| {
            b.target == target
                && b.params == *params
                && b.planes_version == planes_version
                && b.thinning == thinning
        });
        (!done).then_some(planes_version)
    }

    /// Records that `target` now holds that blend.
    pub(super) fn remember_bake(
        &self,
        target: BakeId,
        params: &ChannelParams,
        thinning: Thinning,
        planes_version: u64,
    ) {
        let mut baked = self.baked.lock().unwrap_or_else(|p| p.into_inner());
        baked.retain(|b| b.target != target);
        if baked.len() >= REMEMBERED {
            baked.remove(0);
        }
        baked.push(Baked {
            params: *params,
            planes_version,
            thinning,
            target,
        });
    }
}
