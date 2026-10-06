//! GPU checks of the OIT passes and composite: draw order does not change the
//! result, translucent layers cover by 1 - Π(1 - α), and opaque layers hide
//! what is behind them. Skipped (passes, with a message) without a GPU.

use super::{OitState, Variant};
use eframe::egui_wgpu::CallbackResources;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const SIZE: u32 = 4;

/// Full-viewport layers: half-transparent red at depth 0.3 and blue at 0.6,
/// opaque green at 0.9, and half-transparent white at 0.95 (behind green).
const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) layer: u32,
};

@vertex
fn vs_main(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> VertexOutput {
    var depths = array<f32, 4>(0.3, 0.6, 0.9, 0.95);
    let xy = vec2<f32>(f32((v << 1u) & 2u), f32(v & 2u)) * 2.0 - 1.0;
    var out: VertexOutput;
    out.position = vec4<f32>(xy, depths[i], 1.0);
    out.layer = i;
    return out;
}

fn shade(in: VertexOutput) -> vec4<f32> {
    var colors = array<vec4<f32>, 4>(
        vec4<f32>(1.0, 0.0, 0.0, 0.5),
        vec4<f32>(0.0, 0.0, 1.0, 0.5),
        vec4<f32>(0.0, 1.0, 0.0, 1.0),
        vec4<f32>(1.0, 1.0, 1.0, 0.5),
    );
    return colors[in.layer];
}

@fragment
fn fs_opaque(in: VertexOutput) -> @location(0) vec4<f32> {
    return oit_opaque(shade(in));
}

@fragment
fn fs_oit(in: VertexOutput) -> OitOutput {
    return oit_accumulate(shade(in), in.position.z);
}
"#;

fn gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(async {
        let adapter = wgpu::Instance::default()
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok()?;
        if !super::supported(&adapter) {
            return None;
        }
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .ok()
    })
}

struct Scene {
    state: OitState,
    empty: wgpu::BindGroup,
    resources: CallbackResources,
}

fn scene(device: &wgpu::Device, queue: &wgpu::Queue) -> Scene {
    let source = format!("{}\n{SHADER}", include_str!("../shaders/common/oit.wgsl"));
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
    let build = |variant: &Variant<'_>| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(variant.label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(variant.entry),
                targets: variant.targets,
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(crate::plots::common::default_depth_stencil_state(
                variant.depth_write,
                wgpu::CompareFunction::LessEqual,
            )),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    };
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

/// Renders the layers in `order`, composites them over a transparent target
/// and returns the first pixel.
fn render(device: &wgpu::Device, queue: &wgpu::Queue, scene: &mut Scene, order: &[u32]) -> [u8; 4] {
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
    scene
        .state
        .render(device, &mut encoder, [SIZE; 2], atlas, |pass| {
            pass.set_bind_group(0, empty, &[]);
            for &layer in order {
                pass.draw(0..3, layer..layer + 1);
            }
        });
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

fn near(a: u8, b: f32) -> bool {
    (f32::from(a) - b).abs() <= 2.0
}

#[test]
fn oit_is_order_independent_and_hides_behind_opaque() {
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIPPED oit_is_order_independent_and_hides_behind_opaque: no OIT-capable GPU");
        return;
    };
    let mut scene = scene(&device, &queue);
    let reference = render(&device, &queue, &mut scene, &[0, 1, 2, 3]);
    for order in [[3, 2, 1, 0], [1, 3, 0, 2], [2, 0, 3, 1]] {
        let pixel = render(&device, &queue, &mut scene, &order);
        for (c, (p, r)) in pixel.iter().zip(reference).enumerate() {
            assert!(
                p.abs_diff(r) <= 1,
                "order {order:?} channel {c}: {p} vs {r}"
            );
        }
    }
    // Red and blue (equal weights this close) cover 1 - 0.5 * 0.5 = 0.75 over
    // opaque green; the white layer behind green adds nothing.
    let [r, g, b, a] = reference;
    assert!(near(r, 0.375 * 255.0), "red {r}");
    assert!(near(b, 0.375 * 255.0), "blue {b}");
    assert!(near(g, 0.25 * 255.0), "green {g}");
    assert!(near(a, 255.0), "alpha {a}");
}

#[test]
fn oit_without_opaque_layers_keeps_coverage() {
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIPPED oit_without_opaque_layers_keeps_coverage: no OIT-capable GPU");
        return;
    };
    let mut scene = scene(&device, &queue);
    // Red alone: premultiplied (0.5, 0, 0, 0.5).
    let [r, g, b, a] = render(&device, &queue, &mut scene, &[0]);
    assert!(near(r, 127.5) && near(g, 0.0) && near(b, 0.0) && near(a, 127.5));
}
