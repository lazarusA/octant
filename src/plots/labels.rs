//! Label images (OME-Zarr segmentations), drawn over a heatmap renderer's
//! data: one color per label id, from the image's `image-label` colors or a
//! stable hashed color, with label 0 transparent; filled or as boundaries.
//!
//! The ids stay exact: label blocks store them as the bits of `f32`s
//! (`zarr/slice.rs` `retrieve_array_subset_as_label_bits`), which
//! `labels.wgsl` reads back as `u32`.

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};

use super::heatmap::HeatmapRenderer;
pub use super::label_palette::{label_color, label_hash, palette_entries};
use crate::data::ome::OmeLabelInfo;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
struct LabelParams {
    count: u32,
    outline: u32,
    opacity: f32,
    _pad: u32,
}

/// The label shader: the shared data quad (`common/quad.wgsl`) in label colors.
pub(crate) const LABELS_WGSL: &str = concat!(
    include_str!("shaders/common/coords.wgsl"),
    "\n",
    include_str!("shaders/common/quad.wgsl"),
    "\n",
    include_str!("shaders/labels.wgsl"),
);

/// The pipeline and label colors of one label layer.
pub struct LabelRenderer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    params_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    count: u32,
}

/// A label layer drawn by a `HeatmapCallback`, over its renderer's data.
pub struct LabelDraw {
    pub renderer: Arc<LabelRenderer>,
    pub outline: bool,
    pub opacity: f32,
}

impl LabelRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        info: &OmeLabelInfo,
    ) -> Self {
        let layout = label_bind_group_layout(device);
        let data_layout = super::common::create_plot_with_coords_bind_group_layout(
            device,
            "Label Data Bind Group Layout",
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
        );
        let pipeline = super::heatmap::quad_pipeline(
            device,
            target_format,
            "Label",
            LABELS_WGSL,
            &[Some(&data_layout), Some(&layout)],
        );
        Self::with_pipeline(device, pipeline, layout, info)
    }

    /// The same pipeline with `info`'s colors.
    pub fn with_palette(&self, device: &wgpu::Device, info: &OmeLabelInfo) -> Self {
        Self::with_pipeline(device, self.pipeline.clone(), self.layout.clone(), info)
    }

    fn with_pipeline(
        device: &wgpu::Device,
        pipeline: wgpu::RenderPipeline,
        layout: wgpu::BindGroupLayout,
        info: &OmeLabelInfo,
    ) -> Self {
        let mut entries = palette_entries(info);
        let count = entries.len() as u32;
        if entries.is_empty() {
            // Storage buffers can't be empty; `count` keeps this one unread.
            entries.push([0, 0]);
        }
        let params_buffer = super::common::create_uniform_buffer(
            device,
            "Label Params Buffer",
            &LabelParams {
                count,
                ..LabelParams::default()
            },
        );
        let palette_buffer =
            super::common::create_storage_buffer(device, "Label Palette Buffer", &entries);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Label Bind Group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: palette_buffer.as_entire_binding(),
                },
            ],
        });
        Self {
            pipeline,
            layout,
            params_buffer,
            bind_group,
            count,
        }
    }

    pub(crate) fn update_params(&self, queue: &wgpu::Queue, outline: bool, opacity: f32) {
        let params = LabelParams {
            count: self.count,
            outline: u32::from(outline),
            opacity,
            _pad: 0,
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
    }

    /// Draws `heatmap`'s ids in their label colors.
    pub(crate) fn draw(&self, rpass: &mut wgpu::RenderPass<'static>, heatmap: &HeatmapRenderer) {
        rpass.set_pipeline(&self.pipeline);
        heatmap.bind_quad(rpass);
        rpass.set_bind_group(1, &self.bind_group, &[]);
        heatmap.draw_quad(rpass);
    }
}

fn label_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let buffer = |binding, ty| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Label Bind Group Layout"),
        entries: &[
            buffer(0, wgpu::BufferBindingType::Uniform),
            buffer(1, wgpu::BufferBindingType::Storage { read_only: true }),
        ],
    })
}
