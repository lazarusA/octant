//! Multi-channel composites blended on the GPU: a block's channel planes
//! are uploaded once, and its channels' tints, windows and curves are
//! uniforms, so a display change costs no CPU blend and no upload. Heatmaps
//! draw them over their quad, as label images are (`labels.rs`); any
//! pipeline with `bind_group_layout` at group 2 blends them through
//! `common/channel_blend.wgsl` `blend_cell`, which mirrors
//! `ChannelAccumulator`.

use std::sync::Arc;

use super::channel_params::{ChannelParams, MAX_CHANNELS};
use super::heatmap::HeatmapRenderer;
use crate::data::slicing::CompositeStretch;

/// The channel shader: the shared data quad (`common/quad.wgsl`) blended.
pub(crate) const CHANNELS_WGSL: &str = concat!(
    include_str!("shaders/common/coords.wgsl"),
    "\n",
    include_str!("shaders/common/quad.wgsl"),
    "\n",
    include_str!("shaders/common/channel_blend.wgsl"),
    "\n",
    include_str!("shaders/channels.wgsl"),
);

/// The quad pipeline and one block's channel planes, bound at group 2.
pub struct ChannelRenderer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    params_buffer: wgpu::Buffer,
    planes_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    /// Cells per plane and planes held.
    pub plane_len: usize,
    pub planes: usize,
}

/// A layer's composite drawn by a `HeatmapCallback`, over its renderer's quad.
pub struct ChannelDraw {
    pub renderer: Arc<ChannelRenderer>,
    pub params: ChannelParams,
}

impl ChannelRenderer {
    /// Whether `planes` planes of `plane_len` cells fit in one storage
    /// buffer (`limits::max_storage_bytes`: the device's, within the
    /// browser's budget there).
    pub fn fits(planes: usize, plane_len: usize) -> bool {
        let bytes = planes.saturating_mul(plane_len).saturating_mul(4);
        (1..=MAX_CHANNELS).contains(&planes) && bytes <= super::limits::max_storage_bytes()
    }

    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        planes: usize,
        plane_len: usize,
    ) -> Self {
        let layout = bind_group_layout(device);
        let data_layout = super::common::create_plot_with_coords_bind_group_layout(
            device,
            "Channel Data Bind Group Layout",
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
        );
        let pipeline = super::quad::quad_pipeline(
            device,
            target_format,
            "Channels",
            CHANNELS_WGSL,
            &[Some(&data_layout), None, Some(&layout)],
        );
        Self::with_pipeline(device, pipeline, layout, planes, plane_len)
    }

    /// Keeps `slot` holding `planes` planes of `plane_len` cells: the
    /// pipeline is built once, the planes buffer again when its size changes.
    pub fn sync(
        slot: &mut Option<Arc<Self>>,
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        planes: usize,
        plane_len: usize,
    ) -> Arc<Self> {
        let renderer = match slot.take() {
            Some(old) if old.planes == planes && old.plane_len == plane_len => old,
            Some(old) => Arc::new(Self::with_pipeline(
                device,
                old.pipeline.clone(),
                old.layout.clone(),
                planes,
                plane_len,
            )),
            None => Arc::new(Self::new(device, target_format, planes, plane_len)),
        };
        *slot = Some(renderer.clone());
        renderer
    }

    fn with_pipeline(
        device: &wgpu::Device,
        pipeline: wgpu::RenderPipeline,
        layout: wgpu::BindGroupLayout,
        planes: usize,
        plane_len: usize,
    ) -> Self {
        let (params_buffer, planes_buffer, bind_group) =
            channel_buffers(device, &layout, planes * plane_len);
        Self {
            pipeline,
            layout,
            params_buffer,
            planes_buffer,
            bind_group,
            plane_len,
            planes,
        }
    }

    /// Uploads plane `plane` (`plane_len` raw values).
    pub fn write_plane(&self, queue: &wgpu::Queue, plane: usize, values: &[f32]) {
        if plane < self.planes && values.len() == self.plane_len {
            let offset = (plane * self.plane_len * 4) as u64;
            queue.write_buffer(&self.planes_buffer, offset, bytemuck::cast_slice(values));
        }
    }

    pub(crate) fn update_params(&self, queue: &wgpu::Queue, params: &ChannelParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(params));
    }

    /// Draws `heatmap`'s quad blended from the planes.
    pub(crate) fn draw(&self, rpass: &mut wgpu::RenderPass<'static>, heatmap: &HeatmapRenderer) {
        rpass.set_pipeline(&self.pipeline);
        heatmap.bind_quad(rpass);
        self.bind(rpass);
        heatmap.draw_quad(rpass);
    }

    /// Binds the parameters and planes at group 2.
    pub(crate) fn bind(&self, rpass: &mut wgpu::RenderPass<'_>) {
        rpass.set_bind_group(2, &self.bind_group, &[]);
    }
}

/// Group 2 for pipelines that blend channels but draw none: no channel
/// drawn (`channel_value` returns the data).
pub(crate) fn empty_bind_group(device: &wgpu::Device) -> wgpu::BindGroup {
    channel_buffers(device, &bind_group_layout(device), 1).2
}

/// Zeroed parameters (no channel drawn) and a planes buffer of `cells`
/// values, bound by `layout`.
fn channel_buffers(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    cells: usize,
) -> (wgpu::Buffer, wgpu::Buffer, wgpu::BindGroup) {
    let params = ChannelParams::of(&[], &CompositeStretch::default(), (0, 0), 0, None);
    let params_buffer =
        super::common::create_uniform_buffer(device, "Channel Params Buffer", &params);
    let planes_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Channel Planes Buffer"),
        // Storage buffers can't be empty.
        size: (cells.max(1) * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Channel Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: planes_buffer.as_entire_binding(),
            },
        ],
    });
    (params_buffer, planes_buffer, bind_group)
}

/// Group 2 of every pipeline that blends channels (`channel_blend.wgsl`).
pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let buffer = |binding, ty| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Channel Bind Group Layout"),
        entries: &[
            buffer(0, wgpu::BufferBindingType::Uniform),
            buffer(1, wgpu::BufferBindingType::Storage { read_only: true }),
        ],
    })
}
