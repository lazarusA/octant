//! GPU/CPU parity: renders WGSL `sample_colormap` from the real atlas into a
//! 256×1 target and compares every pixel with the CPU `sample_lut`.
//! Skipped (passes) when no GPU adapter is available.

use super::ColormapAtlas;
use crate::utils::colormap::{LUT_SIZE, registry, sample_colormap_rgb};

const PROBE_SHADER: &str = r#"
struct Probe { colormap: u32, _p0: u32, _p1: u32, _p2: u32 };
@group(0) @binding(0) var<uniform> probe: Probe;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let xy = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(xy * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(sample_colormap(probe.colormap, (p.x - 0.5) / 255.0), 1.0);
}
"#;

fn gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(async {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok()?;
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .ok()
    })
}

fn probe_bind_group(device: &wgpu::Device, id: u32) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    use wgpu::util::DeviceExt;
    let probe = [id, 0, 0, 0];
    let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("probe uniform"),
        contents: bytemuck::cast_slice(&probe),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let probe_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let probe_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &probe_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }],
    });
    (probe_layout, probe_group)
}

fn probe_pipeline(
    device: &wgpu::Device,
    probe_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let source = format!(
        "{}\n{PROBE_SHADER}",
        include_str!("shaders/colormaps/mod.wgsl")
    );
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("probe"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let atlas_layout = super::bind_group_layout(device);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(probe_layout), Some(&atlas_layout)],
        immediate_size: 0,
    });
    let format = wgpu::TextureFormat::Rgba8Unorm;
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("probe"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs"),
            targets: &[Some(format.into())],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn row_target(device: &wgpu::Device) -> wgpu::Texture {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: LUT_SIZE as u32,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn readback_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (LUT_SIZE * 4) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn render_row(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    atlas: &ColormapAtlas,
    id: u32,
) -> Vec<u8> {
    let (probe_layout, probe_group) = probe_bind_group(device, id);
    let pipeline = probe_pipeline(device, &probe_layout);
    let target = row_target(device);
    let readback = readback_buffer(device);
    let view = target.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &probe_group, &[]);
        pass.set_bind_group(1, &atlas.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    read_back(device, queue, encoder, &target, &readback)
}

fn read_back(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    target: &wgpu::Texture,
    readback: &wgpu::Buffer,
) -> Vec<u8> {
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((LUT_SIZE * 4) as u32),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: LUT_SIZE as u32,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    readback
        .slice(..)
        .get_mapped_range()
        .map(|view| view.to_vec())
        .unwrap_or_default()
}

#[test]
fn gpu_atlas_matches_cpu_sampling() {
    let Some((device, queue)) = gpu() else {
        eprintln!("no GPU adapter; skipping colormap GPU parity test");
        return;
    };
    let atlas = ColormapAtlas::new(&device, &queue);
    let last = u32::try_from(registry::len() - 1).unwrap_or(0);
    for id in [
        registry::default_id(),
        registry::find("classic:turbo").unwrap_or(0),
        last,
    ] {
        let pixels = render_row(&device, &queue, &atlas, id);
        assert_eq!(pixels.len(), LUT_SIZE * 4, "readback failed");
        for x in 0..LUT_SIZE {
            let cpu = sample_colormap_rgb(id, x as f32 / 255.0);
            let gpu = &pixels[x * 4..x * 4 + 3];
            for (c, (g, e)) in gpu.iter().zip([cpu.r(), cpu.g(), cpu.b()]).enumerate() {
                assert!(
                    g.abs_diff(e) <= 1,
                    "map {id} texel {x} channel {c}: gpu {g} cpu {e}"
                );
            }
        }
    }
}
