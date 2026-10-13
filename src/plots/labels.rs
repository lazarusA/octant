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

/// The pipeline and label colors of one label layer.
pub struct LabelRenderer {
    pipeline: wgpu::RenderPipeline,
    params_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    count: u32,
}

impl LabelRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        info: &OmeLabelInfo,
    ) -> Self {
        let source = concat!(
            include_str!("shaders/common/coords.wgsl"),
            "\n",
            include_str!("shaders/labels.wgsl"),
        );
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Label Shader Module"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
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
        let layout = label_bind_group_layout(device);
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
        let data_layout = super::common::create_plot_with_coords_bind_group_layout(
            device,
            "Label Data Bind Group Layout",
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
        );
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Label Pipeline Layout"),
            bind_group_layouts: &[Some(&data_layout), Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Label Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(super::heatmap::HeatmapVertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(super::common::default_depth_stencil_state(
                false,
                wgpu::CompareFunction::Always,
            )),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
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
}

impl LabelRenderer {
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

impl super::traits::PlotRenderer for LabelRenderer {
    /// Label ids live in the heatmap renderer's data buffer.
    fn update_data(&self, _queue: &wgpu::Queue, _data: &crate::data::RenderData) {}

    fn paint(
        &self,
        _ui: &mut egui::Ui,
        _rect: egui::Rect,
        _params: &super::traits::PlotRenderParams,
    ) {
        // Painted through `LabelCallback`.
    }

    fn inspect_hover(
        &self,
        _pointer_pos: egui::Pos2,
        _rect: egui::Rect,
        _data: &crate::data::RenderData,
        _params: &super::traits::PlotRenderParams,
    ) -> Option<super::traits::HoverSample> {
        None
    }
}

/// Paints a label layer: the heatmap renderer's ids through `labels`.
pub struct LabelCallback {
    pub heatmap: Arc<HeatmapRenderer>,
    pub labels: Arc<LabelRenderer>,
    pub rect: egui::Rect,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub aspect_scale: [f32; 2],
    pub coord_mode: u32,
    pub outline: bool,
    pub opacity: f32,
}

impl eframe::egui_wgpu::CallbackTrait for LabelCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        self.heatmap.update_uniforms(
            queue,
            &super::common::PlotColorParams::default(),
            self.pan,
            self.zoom,
            self.aspect_scale,
            self.coord_mode,
        );
        self.labels.update_params(queue, self.outline, self.opacity);
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        rpass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        if !super::common::setup_viewport_and_scissor(rpass, &self.rect, &info) {
            return;
        }
        self.labels.draw(rpass, &self.heatmap);
    }
}
