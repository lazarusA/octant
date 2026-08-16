use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct VolumeVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

impl VolumeVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

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
pub struct VolumeUniforms {
    pub clip_planes: [[f32; 4]; 8],
    pub light_color: [f32; 3],
    pub num_clip_planes: u32,
    pub ambient: [f32; 3],
    pub shininess: f32,
    pub light_direction: [f32; 3],
    pub algorithm: u32,
    pub isovalue: f32,
    pub isorange: f32,
    pub absorption: f32,
    pub samples: u32,
    pub diffuse: f32,
    pub specular: f32,
    pub depth_shift: f32,
    pub picking: u32,
    pub object_id: u32,
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_x: f32,
    pub aspect_y: f32,
    pub aspect_z: f32,
    pub zoom: f32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub screen_aspect: f32,
    pub shift_x: u32,
    pub shift_y: u32,
    pub shift_z: u32,
    pub transparency: u32,
    pub _pad1: u32,
    pub color: super::common::PlotColorParams,
}

pub struct VolumeRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub uniform_buffer: wgpu::Buffer,
    pub data_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub data_len: AtomicUsize,
}

impl VolumeRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        initial_data: &[f32],
        width: u32,
        height: u32,
    ) -> Self {
        let shader_source = crate::assemble_plot_shader!(include_str!("shaders/volume.wgsl"));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Volume Raymarching Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let initial_data_safe = if initial_data.is_empty() {
            vec![50.0; 64 * 64 * 16]
        } else {
            initial_data.to_vec()
        };

        let data_len = initial_data_safe.len();
        let depth = super::common::calculate_3d_depth(data_len, width, height);

        let initial_uniforms = VolumeUniforms {
            clip_planes: [[0.0; 4]; 8],
            light_color: [1.0, 1.0, 1.0],
            num_clip_planes: 0,
            ambient: [0.2, 0.2, 0.2],
            shininess: 32.0,
            light_direction: [1.0, 1.0, 1.0],
            algorithm: 0,
            isovalue: 50.0,
            isorange: 5.0,
            absorption: 2.0,
            samples: 64,
            diffuse: 0.8,
            specular: 0.2,
            depth_shift: 0.0,
            picking: 0,
            object_id: 0,
            rotation_y: 0.0,
            rotation_x: 0.0,
            aspect_x: 1.0,
            aspect_y: 1.0,
            aspect_z: 1.0,
            zoom: 2.5,
            width: width.max(1),
            height: height.max(1),
            depth,
            screen_aspect: 1.0,
            shift_x: 0,
            shift_y: 0,
            shift_z: 0,
            transparency: 1,
            _pad1: 0,
            color: super::common::PlotColorParams::default(),
        }; // 3D Bounding Box Vertices [-1, 1]^3
        let vertices = [
            // Front face
            VolumeVertex {
                position: [-1.0, -1.0, 1.0],
                uv: [0.0, 0.0],
            },
            VolumeVertex {
                position: [1.0, -1.0, 1.0],
                uv: [1.0, 0.0],
            },
            VolumeVertex {
                position: [1.0, 1.0, 1.0],
                uv: [1.0, 1.0],
            },
            VolumeVertex {
                position: [-1.0, 1.0, 1.0],
                uv: [0.0, 1.0],
            },
            // Back face
            VolumeVertex {
                position: [-1.0, -1.0, -1.0],
                uv: [0.0, 0.0],
            },
            VolumeVertex {
                position: [1.0, -1.0, -1.0],
                uv: [1.0, 0.0],
            },
            VolumeVertex {
                position: [1.0, 1.0, -1.0],
                uv: [1.0, 1.0],
            },
            VolumeVertex {
                position: [-1.0, 1.0, -1.0],
                uv: [0.0, 1.0],
            },
        ];

        let indices: [u16; 36] = [
            0, 1, 2, 0, 2, 3, // Front
            5, 4, 7, 5, 7, 6, // Back
            4, 0, 3, 4, 3, 7, // Left
            1, 5, 6, 1, 6, 2, // Right
            3, 2, 6, 3, 6, 7, // Top
            4, 5, 1, 4, 1, 0, // Bottom
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Volume Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Volume Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let uniform_buffer = super::common::create_uniform_buffer(
            device,
            "Volume Uniform Buffer",
            &initial_uniforms,
        );

        let data_buffer = super::common::create_storage_buffer(
            device,
            "Volume Data Storage Buffer",
            &initial_data_safe,
        );

        let bind_group_layout = super::common::create_uniform_storage_bind_group_layout(
            device,
            "Volume Bind Group Layout",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );

        let bind_group = super::common::create_uniform_storage_bind_group(
            device,
            "Volume Bind Group",
            &bind_group_layout,
            &uniform_buffer,
            &data_buffer,
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Volume Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Volume Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(VolumeVertex::desc())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
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
            depth_stencil: Some(super::common::default_depth_stencil_state(
                false,
                wgpu::CompareFunction::Always,
            )),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
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
            data_len: AtomicUsize::new(data_len),
        }
    }

    pub fn update_data(&self, queue: &wgpu::Queue, data: &[f32]) {
        if !data.is_empty() {
            queue.write_buffer(&self.data_buffer, 0, bytemuck::cast_slice(data));
            self.data_len.store(data.len(), Ordering::Relaxed);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        color: &super::common::PlotColorParams,
        rot_y: f32,
        rot_x: f32,
        aspect_x: f32,
        aspect_y: f32,
        aspect_z: f32,
        zoom: f32,
        opacity_scale: f32,
        step_count: u32,
        width: u32,
        height: u32,
        algorithm: u32,
        isovalue: f32,
        isorange: f32,
        screen_aspect: f32,
        shift_x: u32,
        shift_y: u32,
        shift_z: u32,
        transparency: bool,
    ) {
        let data_l = self.data_len.load(Ordering::Relaxed);
        let depth = super::common::calculate_3d_depth(data_l, width, height);
        let uniforms = VolumeUniforms {
            clip_planes: [[0.0; 4]; 8],
            light_color: [1.0, 1.0, 1.0],
            num_clip_planes: 0,
            ambient: [0.2, 0.2, 0.2],
            shininess: 32.0,
            light_direction: [1.0, 1.0, 1.0],
            algorithm,
            isovalue,
            isorange,
            absorption: opacity_scale,
            samples: step_count,
            diffuse: 0.8,
            specular: 0.2,
            depth_shift: 0.0,
            picking: 0,
            object_id: 0,
            rotation_y: rot_y,
            rotation_x: rot_x,
            aspect_x,
            aspect_y,
            aspect_z,
            zoom,
            width: width.max(1),
            height: height.max(1),
            depth,
            screen_aspect,
            shift_x,
            shift_y,
            shift_z,
            transparency: if transparency { 1 } else { 0 },
            _pad1: 0,
            color: *color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }
}

impl super::common::PlotRenderer for VolumeRenderer {
    fn update_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        VolumeRenderer::update_data(self, queue, values);
    }
}

pub struct VolumeCallback {
    pub renderer: Arc<VolumeRenderer>,
    pub color_params: super::common::PlotColorParams,
    pub rot_y: f32,
    pub rot_x: f32,
    pub aspect_x: f32,
    pub aspect_y: f32,
    pub aspect_z: f32,
    pub zoom: f32,
    pub opacity_scale: f32,
    pub step_count: u32,
    pub width: u32,
    pub height: u32,
    pub algorithm: u32,
    pub isovalue: f32,
    pub isorange: f32,
    pub shift_x: u32,
    pub shift_y: u32,
    pub shift_z: u32,
    pub transparency: bool,
    pub rect: egui::Rect,
}

impl eframe::egui_wgpu::CallbackTrait for VolumeCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let screen_aspect = super::common::compute_aspect_ratio(&self.rect);
        self.renderer.update_uniforms(
            queue,
            &self.color_params,
            self.rot_y,
            self.rot_x,
            self.aspect_x,
            self.aspect_y,
            self.aspect_z,
            self.zoom,
            self.opacity_scale,
            self.step_count,
            self.width,
            self.height,
            self.algorithm,
            self.isovalue,
            self.isorange,
            screen_aspect,
            self.shift_x,
            self.shift_y,
            self.shift_z,
            self.transparency,
        );
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        rpass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        super::common::setup_viewport_and_scissor(rpass, &self.rect, &info);

        rpass.set_pipeline(&self.renderer.render_pipeline);
        rpass.set_bind_group(0, &self.renderer.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
        rpass.set_index_buffer(
            self.renderer.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        rpass.draw_indexed(0..self.renderer.index_count, 0, 0..1);
    }
}
