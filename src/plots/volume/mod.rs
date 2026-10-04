//! Volume raymarcher: voxel values in 3D textures, sampled trilinearly on a
//! fixed step per voxel with jittered starts and step-independent opacity,
//! rendered into a cached offscreen frame.
//!
//! - `encode.rs`: CPU plane encoding (missing-voxel fill, validity).
//! - `textures.rs`: value, validity and brick 3D textures, plane uploads.
//! - `bricks.rs`: empty-space skipping grid (per-brick value range).
//! - `lut.rs`: transfer-function lookup texture from the colormap registry.
//! - `uniforms.rs`: uniform block and per-frame parameters.
//! - `pipeline.rs`: shader variants, bind group layout, per-mode pipelines.
//! - `frame.rs`: offscreen frame, its cache key and the blit.
//! - `render.rs`: re-renders the frame when its key changes, blits it.
//! - `callback.rs`: egui paint callback.

mod bricks;
mod callback;
pub mod encode;
mod frame;
pub mod lut;
pub mod pipeline;
mod render;
mod textures;
mod uniforms;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod gpu_render;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod gpu_tests;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod sheet;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod skip_tests;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod slab_sheet;
#[cfg(test)]
mod tests;

pub use callback::VolumeCallback;
pub use encode::VolumeEncoding;
pub use uniforms::{VolumeState, VolumeUniformParams, VolumeUniforms};

use bytemuck::Zeroable;
use encode::Dims;
use frame::Blit;
use lut::TransferLut;
use render::RenderCache;
use std::sync::Mutex;
use textures::VolumeTextures;
use wgpu::util::DeviceExt;

pub struct VolumeRenderer {
    module: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    textures: VolumeTextures,
    transfer: TransferLut,
    blit: Blit,
    cache: Mutex<RenderCache>,
}

impl VolumeRenderer {
    /// Creates the renderer for a `width`×`height`×(`data.len()` / plane) volume,
    /// or `None` (logged) when it does not fit the device's 3D texture limit.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        data: &[f32],
        width: u32,
        height: u32,
        encoding: VolumeEncoding,
    ) -> Option<Self> {
        let hardware_filter = pipeline::hardware_filtering(device);
        let geometry = (data, width, height);
        Self::build(
            device,
            queue,
            target_format,
            geometry,
            encoding,
            hardware_filter,
        )
    }

    /// [`Self::new`] with the filtering variant chosen by the caller.
    fn build(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        (data, width, height): (&[f32], u32, u32),
        encoding: VolumeEncoding,
        hardware_filter: bool,
    ) -> Option<Self> {
        let (w, h) = (width.max(1) as usize, height.max(1) as usize);
        let plane = w.checked_mul(h)?;
        let dims = Dims {
            w,
            h,
            d: (data.len() / plane).max(1),
        };
        let textures = VolumeTextures::new(device, dims, encoding)?;
        let layout = pipeline::bind_group_layout(device, hardware_filter);
        let transfer = TransferLut::new(device);
        let sampler = pipeline::create_sampler(device);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Volume Uniform Buffer"),
            contents: bytemuck::bytes_of(&VolumeUniforms::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Volume Bind Group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                view_entry(1, &textures.value_view),
                view_entry(2, &textures.validity_view),
                view_entry(3, &transfer.view),
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                view_entry(5, &textures.bricks_view),
            ],
        });
        let renderer = Self {
            module: pipeline::create_module(device, hardware_filter),
            pipeline_layout: pipeline::create_pipeline_layout(device, &layout),
            uniform_buffer,
            bind_group,
            textures,
            transfer,
            blit: Blit::new(device, target_format),
            cache: Mutex::new(RenderCache::default()),
        };
        renderer.update_data(queue, data);
        Some(renderer)
    }

    pub fn encoding(&self) -> VolumeEncoding {
        self.textures.encoding
    }

    /// Uploads the whole volume; `data` must match the texture's voxel count.
    pub fn update_data(&self, queue: &wgpu::Queue, data: &[f32]) {
        self.textures
            .upload_planes(queue, data, 0..self.textures.dims.d);
    }

    /// Uploads Z planes `z` of the full volume `data` (and the planes next to them).
    pub fn update_planes(&self, queue: &wgpu::Queue, data: &[f32], z: std::ops::Range<usize>) {
        self.textures.upload_planes(queue, data, z);
    }

    fn state(&self) -> VolumeState {
        let Dims { w, h, d } = self.textures.dims;
        VolumeState {
            dims: [w as u32, h as u32, d as u32],
            has_invalid: self.textures.has_invalid(),
            composite: self.textures.encoding == VolumeEncoding::PackedRgb,
        }
    }
}

fn view_entry(binding: u32, view: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

impl super::common::PlotRenderer for VolumeRenderer {
    fn update_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        self.update_data(queue, values);
    }
}

impl super::traits::PlotRenderer for VolumeRenderer {
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
