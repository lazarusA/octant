//! The bricked volume renderer (`bricks.rs`): the dense renderer's
//! pipeline over the brick atlas, table, ranges and ray feedback.

use std::sync::Mutex;

use bytemuck::Zeroable;
use wgpu::util::DeviceExt;

use super::bricks::{AtlasTexel, BrickTextures};
use super::frame::Blit;
use super::render::RenderCache;
use super::uniforms::VolumeUniforms;
use super::{VolumeRenderer, pipeline, view_entry};

impl VolumeRenderer {
    /// A bricked volume: an atlas of `slots` per axis and a table of `grid`
    /// cells (`bricks.rs`), drawn by the bricked shader variant; `None` past
    /// the device's 3D texture size.
    pub fn new_bricked(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        (slots, grid, texel): ([u32; 3], [u32; 3], AtlasTexel),
    ) -> Option<Self> {
        let bricks = BrickTextures::new(device, slots, grid, texel)?;
        let layout = bricked_layout(device);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bricked Volume Uniform Buffer"),
            contents: bytemuck::bytes_of(&VolumeUniforms::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = bricked_bind_group(device, &layout, &uniform_buffer, &bricks);
        Some(Self {
            module: pipeline::create_bricked_module(device),
            pipeline_layout: pipeline::create_pipeline_layout(device, &layout),
            uniform_buffer,
            bind_group,
            textures: None,
            blit: Blit::new(device, target_format),
            cache: Mutex::new(RenderCache::default()),
            bricks: Some(bricks),
        })
    }

    /// A bricked volume's atlas and table.
    pub fn bricks(&self) -> Option<&BrickTextures> {
        self.bricks.as_ref()
    }
}

/// `@group(0)` of a bricked volume: the frame uniforms at 0, the brick atlas
/// at 1 (always filterable), the sampler at 3, the bricks' uniforms at 5,
/// their table at 6, slot ranges at 7, ray feedback at 8 and sub-block ranges
/// at 9.
pub(super) fn bricked_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    use wgpu::TextureViewDimension::D3;
    let stage = wgpu::ShaderStages::VERTEX_FRAGMENT;
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bricked Volume Bind Group Layout"),
        entries: &[
            pipeline::uniform_entry(0, stage),
            pipeline::texture_entry(1, true, D3),
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            pipeline::uniform_entry(5, wgpu::ShaderStages::FRAGMENT),
            storage_entry(7, true),
            storage_entry(9, true),
            storage_entry(8, false),
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: D3,
                    multisampled: false,
                },
                count: None,
            },
        ],
    })
}

/// A read-only (or, for the ray feedback, writable) storage buffer read in
/// the fragment stage.
fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// `@group(0)` of a bricked volume over `layout`: the frame's `uniforms` and
/// the bricks' atlas, table and buffers.
fn bricked_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniforms: &wgpu::Buffer,
    bricks: &BrickTextures,
) -> wgpu::BindGroup {
    let sampler = pipeline::create_sampler(device);
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bricked Volume Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            },
            view_entry(1, &bricks.atlas_view),
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: bricks.uniforms.as_entire_binding(),
            },
            view_entry(6, &bricks.table_view),
            wgpu::BindGroupEntry {
                binding: 7,
                resource: bricks.ranges.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: bricks.feedback.seen.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 9,
                resource: bricks.subs.as_entire_binding(),
            },
        ],
    })
}
