use std::sync::atomic::{AtomicU32, Ordering};
use wgpu::util::DeviceExt;

use super::oit::{OitSlot, Variant, build_pipeline, egui_target};
pub use super::point_cloud_draw::PointCloudCallback;
pub use super::point_cloud_types::{PointCloudUniformParams, PointCloudUniforms, PointCloudVertex};

pub struct PointCloudRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    /// Translucent colors: no depth writes.
    pub transparent_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub uniform_buffer: wgpu::Buffer,
    pub data_buffer: wgpu::Buffer,
    /// Names `data_buffer` to the channel bake.
    pub bake_id: super::BakeId,
    /// How `data_buffer` thins the volume, for the channel bake.
    pub thinning: super::Thinning,
    pub bind_group: wgpu::BindGroup,
    pub instance_count: AtomicU32,
    /// Kept to build the OIT pipelines on first use.
    pub(super) shader: wgpu::ShaderModule,
    pub(super) pipeline_layout: wgpu::PipelineLayout,
    pub(crate) oit: OitSlot,
    /// Its frame when drawn without OIT.
    pub(crate) frame: super::frame_cache::FrameSlot,
    /// What its OIT frame depends on.
    pub(crate) inputs: super::oit::FrameInputs,
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
            camera: super::camera::OrbitCamera::default().uniform(),
            aspect_x: 1.0,
            aspect_y: 1.0,
            aspect_z: 1.0,
            point_size: 1.05,
            width: width.max(1),
            height: height.max(1),
            depth,
            shift_x: 0,
            shift_y: 0,
            shift_z: 0,
            scale_by_value: 0,
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

        let target = egui_target(target_format);
        let pipeline = |label, depth_write| {
            let variant = Variant::egui(label, &target, depth_write);
            build_pipeline(
                device,
                &pipeline_layout,
                &shader,
                &[Some(PointCloudVertex::desc())],
                None,
                &variant,
            )
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
            bake_id: super::BakeId::next(),
            thinning: super::Thinning::NONE,
            bind_group,
            instance_count: AtomicU32::new(instance_count),
            shader,
            pipeline_layout,
            oit: OitSlot::new(target_format),
            frame: super::frame_cache::FrameSlot::new(target_format),
            inputs: Default::default(),
        }
    }

    /// This renderer, its data a volume thinned by `thinning`.
    pub fn with_thinning(self, thinning: super::Thinning) -> Self {
        Self { thinning, ..self }
    }

    pub fn update_data(&self, queue: &wgpu::Queue, data: &[f32]) {
        self.inputs.data_changed();
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
        self.inputs.data_changed();
        super::common::safe_write_buffer_range(
            queue,
            &self.data_buffer,
            offset,
            data,
            "PointCloudRenderer::update_data_range",
        );
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
