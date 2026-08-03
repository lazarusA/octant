use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::util::DeviceExt;

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
    pub colormap: u32,
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
    pub _pad0: u32,
}

pub struct PointCloudRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub uniform_buffer: wgpu::Buffer,
    pub data_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub instance_count: u32,
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

        // Unit quad template for point billboard
        let vertices = [
            PointCloudVertex { position: [-0.5, -0.5] },
            PointCloudVertex { position: [ 0.5, -0.5] },
            PointCloudVertex { position: [ 0.5,  0.5] },
            PointCloudVertex { position: [-0.5,  0.5] },
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

        let initial_data_safe = if initial_data.is_empty() { vec![50.0; 64 * 64] } else { initial_data.to_vec() };
        let instance_count = initial_data_safe.len() as u32;
        let depth = (instance_count / (width.max(1) * height.max(1))).max(1);

        let uniforms = PointCloudUniforms {
            colormap: 0,
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
            _pad0: 0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Point Cloud Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Point Cloud Data Storage Buffer"),
            contents: bytemuck::cast_slice(&initial_data_safe),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Point Cloud Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Point Cloud Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: data_buffer.as_entire_binding() },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Point Cloud Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Point Cloud Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[PointCloudVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            uniform_buffer,
            data_buffer,
            bind_group,
            instance_count,
        }
    }

    pub fn update_data(&mut self, queue: &wgpu::Queue, data: &[f32]) {
        if !data.is_empty() && data.len() as u32 == self.instance_count {
            queue.write_buffer(&self.data_buffer, 0, bytemuck::cast_slice(data));
        }
    }

    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        colormap: u32,
        rot_y: f32,
        rot_x: f32,
        aspect_x: f32,
        aspect_y: f32,
        aspect_z: f32,
        zoom: f32,
        point_size: f32,
        width: u32,
        height: u32,
    ) {
        let depth = (self.instance_count / (width.max(1) * height.max(1))).max(1);
        let uniforms = PointCloudUniforms {
            colormap,
            rotation_y: rot_y,
            rotation_x: rot_x,
            aspect_x,
            aspect_y,
            aspect_z,
            zoom,
            point_size,
            width: width.max(1),
            height: height.max(1),
            depth,
            _pad0: 0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }
}

pub struct PointCloudCallback {
    pub renderer: Arc<PointCloudRenderer>,
    pub colormap: u32,
    pub rot_y: f32,
    pub rot_x: f32,
    pub aspect_x: f32,
    pub aspect_y: f32,
    pub aspect_z: f32,
    pub zoom: f32,
    pub point_size: f32,
    pub width: u32,
    pub height: u32,
    pub rect: egui::Rect,
}

impl eframe::egui_wgpu::CallbackTrait for PointCloudCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        self.renderer.update_uniforms(
            queue,
            self.colormap,
            self.rot_y,
            self.rot_x,
            self.aspect_x,
            self.aspect_y,
            self.aspect_z,
            self.zoom,
            self.point_size,
            self.width,
            self.height,
        );
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        rpass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        let ppp = info.pixels_per_point;
        let px_x = (self.rect.min.x * ppp).max(0.0) as u32;
        let px_y = (self.rect.min.y * ppp).max(0.0) as u32;
        let px_w = (self.rect.width() * ppp).max(1.0) as u32;
        let px_h = (self.rect.height() * ppp).max(1.0) as u32;

        rpass.set_viewport(px_x as f32, px_y as f32, px_w as f32, px_h as f32, 0.0, 1.0);
        rpass.set_scissor_rect(px_x, px_y, px_w, px_h);

        rpass.set_pipeline(&self.renderer.render_pipeline);
        rpass.set_bind_group(0, &self.renderer.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.renderer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..self.renderer.index_count, 0, 0..self.renderer.instance_count);
    }
}
