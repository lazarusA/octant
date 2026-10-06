use bytemuck::{Pod, Zeroable};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use wgpu::util::DeviceExt;

use super::oit::{OitState, Variant};
pub use super::point_cloud_draw::PointCloudCallback;
use super::point_cloud_draw::point_cloud_pipeline;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PointCloudVertex {
    pub position: [f32; 2],
}

impl PointCloudVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PointCloudUniforms {
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_x: f32,
    pub aspect_y: f32,
    pub aspect_z: f32,
    pub zoom: f32,
    pub point_size: f32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub screen_aspect: f32,
    pub shift_x: u32,
    pub shift_y: u32,
    pub shift_z: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub color: super::common::PlotColorParams,
}

pub struct PointCloudRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    /// Translucent colors: no depth writes.
    pub transparent_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub uniform_buffer: wgpu::Buffer,
    pub data_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub instance_count: AtomicU32,
    /// Kept to build the OIT pipelines on first use.
    pub(super) shader: wgpu::ShaderModule,
    pub(super) pipeline_layout: wgpu::PipelineLayout,
    pub(super) target_format: wgpu::TextureFormat,
    pub(super) oit: Mutex<Option<OitState>>,
}

impl PointCloudRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        initial_data: &[f32],
        width: u32,
        height: u32,
    ) -> Self {
        let shader_source = crate::assemble_plot_shader!(include_str!("shaders/point_cloud.wgsl"));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Point Cloud Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let initial_data_safe = if initial_data.is_empty() {
            vec![50.0; 64 * 64 * 16]
        } else {
            initial_data.to_vec()
        };

        let instance_count = initial_data_safe.len() as u32;
        let depth = super::common::calculate_3d_depth(initial_data_safe.len(), width, height);

        let initial_uniforms = PointCloudUniforms {
            rotation_y: 0.0,
            rotation_x: 0.0,
            aspect_x: 1.0,
            aspect_y: 1.0,
            aspect_z: 1.0,
            zoom: 2.5,
            point_size: 0.02,
            width: width.max(1),
            height: height.max(1),
            depth,
            screen_aspect: 1.0,
            shift_x: 0,
            shift_y: 0,
            shift_z: 0,
            _pad0: 0,
            _pad1: 0,
            color: super::common::PlotColorParams::default(),
        }; // Unit quad template for point billboard
        let vertices = [
            PointCloudVertex {
                position: [-0.5, -0.5],
            },
            PointCloudVertex {
                position: [0.5, -0.5],
            },
            PointCloudVertex {
                position: [0.5, 0.5],
            },
            PointCloudVertex {
                position: [-0.5, 0.5],
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Point Cloud Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Point Cloud Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let uniform_buffer = super::common::create_uniform_buffer(
            device,
            "Point Cloud Uniform Buffer",
            &initial_uniforms,
        );

        let data_buffer = super::common::create_storage_buffer(
            device,
            "Point Cloud Data Storage Buffer",
            &initial_data_safe,
        );

        let bind_group_layout = super::common::create_uniform_storage_bind_group_layout(
            device,
            "Point Cloud Bind Group Layout",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );

        let bind_group = super::common::create_uniform_storage_bind_group(
            device,
            "Point Cloud Bind Group",
            &bind_group_layout,
            &uniform_buffer,
            &data_buffer,
        );

        let colormap_layout = super::colormap_atlas::bind_group_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Point Cloud Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout), Some(&colormap_layout)],
            immediate_size: 0,
        });

        let egui_target = [Some(wgpu::ColorTargetState {
            format: target_format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let pipeline = |label, depth_write| {
            let variant = Variant {
                label,
                entry: "fs_main",
                targets: &egui_target,
                depth_write,
            };
            point_cloud_pipeline(device, &pipeline_layout, &shader, &variant)
        };
        let render_pipeline = pipeline("Point Cloud Render Pipeline", true);
        // Translucent colors: no depth writes, so no point hides the ones behind it.
        let transparent_pipeline = pipeline("Point Cloud Transparent Render Pipeline", false);

        Self {
            render_pipeline,
            transparent_pipeline,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            uniform_buffer,
            data_buffer,
            bind_group,
            instance_count: AtomicU32::new(instance_count),
            shader,
            pipeline_layout,
            target_format,
            oit: Mutex::new(None),
        }
    }

    pub fn update_data(&self, queue: &wgpu::Queue, data: &[f32]) {
        if !data.is_empty()
            && super::common::safe_write_buffer(
                queue,
                &self.data_buffer,
                data,
                "PointCloudRenderer::update_data",
            )
        {
            self.instance_count
                .store(data.len() as u32, Ordering::Relaxed);
        }
    }

    /// Uploads `data` at element `offset` without touching the rest of the volume.
    pub fn update_data_range(&self, queue: &wgpu::Queue, offset: usize, data: &[f32]) {
        super::common::safe_write_buffer_range(
            queue,
            &self.data_buffer,
            offset,
            data,
            "PointCloudRenderer::update_data_range",
        );
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PointCloudUniformParams {
    pub color: super::common::PlotColorParams,
    pub rot_y: f32,
    pub rot_x: f32,
    pub aspect_x: f32,
    pub aspect_y: f32,
    pub aspect_z: f32,
    pub zoom: f32,
    pub point_size: f32,
    pub width: u32,
    pub height: u32,
    pub screen_aspect: f32,
    pub shift_x: u32,
    pub shift_y: u32,
    pub shift_z: u32,
}

impl PointCloudRenderer {
    pub fn update_uniforms(&self, queue: &wgpu::Queue, params: &PointCloudUniformParams) {
        let instance_cnt = self.instance_count.load(Ordering::Relaxed);
        let depth =
            super::common::calculate_3d_depth(instance_cnt as usize, params.width, params.height);
        let uniforms = PointCloudUniforms {
            rotation_y: params.rot_y,
            rotation_x: params.rot_x,
            aspect_x: params.aspect_x,
            aspect_y: params.aspect_y,
            aspect_z: params.aspect_z,
            zoom: params.zoom,
            point_size: params.point_size,
            width: params.width.max(1),
            height: params.height.max(1),
            depth,
            screen_aspect: params.screen_aspect,
            shift_x: params.shift_x,
            shift_y: params.shift_y,
            shift_z: params.shift_z,
            _pad0: 0,
            _pad1: 0,
            color: params.color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }
}

impl super::common::PlotRenderer for PointCloudRenderer {
    fn update_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        self.update_data(queue, values);
    }
}

impl super::traits::PlotRenderer for PointCloudRenderer {
    fn update_data(&self, queue: &wgpu::Queue, data: &crate::data::RenderData) {
        match data {
            crate::data::RenderData::Volume(v) => self.update_data(queue, &v.values),
            crate::data::RenderData::Matrix(m) => self.update_data(queue, &m.values),
        }
    }

    fn paint(
        &self,
        _ui: &mut egui::Ui,
        _rect: egui::Rect,
        _params: &super::traits::PlotRenderParams,
    ) {
        // Concrete painter dispatched via egui callback
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
