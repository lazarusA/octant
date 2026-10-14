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
    /// The label metadata whose colors `bind_group` holds.
    info: Arc<OmeLabelInfo>,
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
        info: &Arc<OmeLabelInfo>,
    ) -> Self {
        let layout = label_bind_group_layout(device);
        let data_layout = super::common::create_plot_with_coords_bind_group_layout(
            device,
            "Label Data Bind Group Layout",
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
        );
        let pipeline = super::quad::quad_pipeline(
            device,
            target_format,
            "Label",
            LABELS_WGSL,
            &[Some(&data_layout), Some(&layout)],
        );
        Self::with_pipeline(device, pipeline, layout, info)
    }

    /// The same pipeline with `info`'s colors.
    pub fn with_palette(&self, device: &wgpu::Device, info: &Arc<OmeLabelInfo>) -> Self {
        Self::with_pipeline(device, self.pipeline.clone(), self.layout.clone(), info)
    }

    /// Keeps `slot` drawing `info`: the pipeline is built once, and the
    /// palette again only when the label metadata changes (not per level).
    pub fn sync(
        slot: &mut Option<Arc<Self>>,
        info: Option<&Arc<OmeLabelInfo>>,
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
    ) {
        *slot = match (info, slot.take()) {
            (None, _) => None,
            (Some(info), Some(old)) if Arc::ptr_eq(&old.info, info) => Some(old),
            (Some(info), Some(old)) => Some(Arc::new(old.with_palette(device, info))),
            (Some(info), None) => Some(Arc::new(Self::new(device, target_format, info))),
        };
    }

    fn with_pipeline(
        device: &wgpu::Device,
        pipeline: wgpu::RenderPipeline,
        layout: wgpu::BindGroupLayout,
        info: &Arc<OmeLabelInfo>,
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
            info: Arc::clone(info),
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

// Label colors on the CPU, exactly as `labels.wgsl` draws them: the palette
// uploaded to the GPU, and the color of one label for the hover.

/// `(id, RGBA packed red-first)` of every color with a 32-bit id, sorted.
pub fn palette_entries(info: &OmeLabelInfo) -> Vec<[u32; 2]> {
    info.colors
        .iter()
        .filter_map(|&(id, rgba)| Some([u32::try_from(id).ok()?, u32::from_le_bytes(rgba)]))
        .collect()
}

/// PCG hash of a label id, as `labels.wgsl` computes it.
fn label_hash(v: u32) -> u32 {
    let state = v.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277_803_737);
    (word >> 22) ^ word
}

/// The color `labels.wgsl` draws label `id` with (before opacity): its
/// recorded color, else one from its hash; `None` for the background (0).
pub fn label_color(info: &OmeLabelInfo, id: u32) -> Option<[u8; 4]> {
    if id == 0 {
        return None;
    }
    if let Some(rgba) = info.color_of(u64::from(id)) {
        return Some(rgba);
    }
    let hue = (label_hash(id) & 0xFFFF) as f32 / 65535.0;
    let (s, v) = (0.65f32, 0.95f32);
    let channel = |k: f32| {
        let p = (((hue + k / 6.0).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0);
        let c = v * (1.0 + (p - 1.0) * s);
        (c * 255.0).round().clamp(0.0, 255.0) as u8
    };
    Some([channel(5.0), channel(3.0), channel(1.0), 255])
}
