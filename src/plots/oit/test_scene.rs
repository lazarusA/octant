//! The OIT tests' scene: five full-viewport layers drawn through an
//! [`OitState`] and read back as one pixel.

use super::{OitState, Variant, build_pipeline};
use eframe::egui_wgpu::CallbackResources;

pub(super) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(super) const SIZE: u32 = 4;

/// Full-viewport layers: half-transparent red at depth 0.3 and blue at 0.6,
/// opaque green at 0.9, half-transparent white at 0.95 (behind green), and
/// half-transparent mid gray at 0.2 (right in front of the camera).
const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) layer: u32,
};

@vertex
fn vs_main(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> VertexOutput {
    var depths = array<f32, 5>(0.3, 0.6, 0.9, 0.95, 0.2);
    let xy = vec2<f32>(f32((v << 1u) & 2u), f32(v & 2u)) * 2.0 - 1.0;
    var out: VertexOutput;
    out.position = vec4<f32>(xy, depths[i], 1.0);
    out.layer = i;
    return out;
}

fn shade(in: VertexOutput) -> vec4<f32> {
    var colors = array<vec4<f32>, 5>(
        vec4<f32>(1.0, 0.0, 0.0, 0.5),
        vec4<f32>(0.0, 0.0, 1.0, 0.5),
        vec4<f32>(0.0, 1.0, 0.0, 1.0),
        vec4<f32>(1.0, 1.0, 1.0, 0.5),
        vec4<f32>(0.5, 0.5, 0.5, 0.5),
    );
    return colors[in.layer];
}

@fragment
fn fs_opaque(in: VertexOutput) -> @location(0) vec4<f32> {
    return oit_opaque(shade(in));
}

@fragment
fn fs_oit(in: VertexOutput) -> OitOutput {
    var cam: Camera;
    cam.near = 0.01;
    cam.far = 50.0;
    return oit_accumulate(shade(in), in.position.z, cam);
}
"#;

/// A device whose adapter can render OIT frames.
pub(super) fn gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    crate::plots::test_gpu::device_with(|adapter| {
        super::supported(adapter).then_some(wgpu::Features::empty())
    })
}

pub(super) struct Scene {
    pub(super) state: OitState,
    empty: wgpu::BindGroup,
    pub(super) resources: CallbackResources,
}

pub(super) fn scene(device: &wgpu::Device, queue: &wgpu::Queue) -> Scene {
    let source = format!(
        "{}\n{}\n{SHADER}",
        include_str!("../shaders/common/camera3d.wgsl"),
        include_str!("../shaders/common/oit.wgsl")
    );
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("oit test"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[],
    });
    let atlas_layout = crate::plots::colormap_atlas::bind_group_layout(device);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&empty_layout), Some(&atlas_layout)],
        immediate_size: 0,
    });
    let build =
        |variant: &Variant<'_>| build_pipeline(device, &layout, &module, &[], None, variant);
    let mut resources = CallbackResources::default();
    crate::plots::colormap_atlas::prepare(device, queue, &mut resources);
    Scene {
        state: OitState::new(device, FORMAT, build),
        empty: device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &empty_layout,
            entries: &[],
        }),
        resources,
    }
}

/// A frame key per layer order, so each order renders.
pub(super) fn key_of(order: &[u32]) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    std::hash::Hash::hash(order, &mut h);
    std::hash::Hasher::finish(&h)
}

/// Renders the layers in `order`, composites them over a transparent target
/// and returns the first pixel.
pub(super) fn render(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scene: &mut Scene,
    order: &[u32],
) -> [u8; 4] {
    let target = |format, usage| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: SIZE,
                height: SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    let color = target(
        FORMAT,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    );
    let depth = target(super::DEPTH_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT);
    let Some(atlas) = crate::plots::colormap_atlas::bind_group(&scene.resources) else {
        panic!("atlas missing");
    };
    let mut encoder = device.create_command_encoder(&Default::default());
    let empty = &scene.empty;
    scene.state.render(
        device,
        &mut encoder,
        ([SIZE; 2], key_of(order)),
        atlas,
        |pass| {
            pass.set_bind_group(0, empty, &[]);
            for &layer in order {
                pass.draw(0..3, layer..layer + 1);
            }
        },
    );
    {
        let color_view = color.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());
        let mut pass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            })
            .forget_lifetime();
        assert!(scene.state.paint(&mut pass), "no frame rendered");
    }
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 256 * u64::from(SIZE),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        color.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(SIZE),
            },
        },
        wgpu::Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let data = slice
        .get_mapped_range()
        .map(|view| view.to_vec())
        .unwrap_or_default();
    std::array::from_fn(|i| data.get(i).copied().unwrap_or_default())
}

pub(super) fn near(a: u8, b: f32) -> bool {
    (f32::from(a) - b).abs() <= 2.0
}
