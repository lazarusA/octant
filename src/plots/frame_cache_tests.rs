//! A cached plot frame draws once per key, and brings its depth into the
//! pass it is drawn in, so what is drawn after it behind the plot stays
//! hidden (3D coastlines) and what is in front shows.

use super::frame_cache::FrameSlot;
use super::labels_tests::{SIZE, read_back, texture};

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// A full-viewport triangle at depth `z` in `rgba` drawn into `targets`,
/// depth-tested; written into depth when `writes`.
fn flat(
    device: &wgpu::Device,
    (layouts, targets): (
        &[Option<&wgpu::BindGroupLayout>],
        &[Option<wgpu::ColorTargetState>],
    ),
    z: f32,
    rgba: &str,
    writes: bool,
) -> wgpu::RenderPipeline {
    let source = format!(
        "@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {{
            let xy = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u)) * 2.0 - 1.0;
            return vec4<f32>(xy, {z:?}, 1.0);
        }}
        @fragment fn fs() -> @location(0) vec4<f32> {{ return vec4<f32>({rgba}); }}"
    );
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: layouts,
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
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
            targets,
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: Some(super::common::default_depth_stencil_state(
            writes,
            wgpu::CompareFunction::LessEqual,
        )),
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}

/// The first pixel after painting `slot`'s frame and then `after`, when
/// given, into a fresh color target cleared to `clear` and depth target.
fn paint_then(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    slot: &FrameSlot,
    (after, clear): (Option<&wgpu::RenderPipeline>, wgpu::Color),
) -> [u8; 4] {
    let usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
    let color = texture(device, FORMAT, usage | wgpu::TextureUsages::COPY_SRC);
    let depth = texture(device, super::oit::DEPTH_FORMAT, usage);
    let (color_view, depth_view) = (
        color.create_view(&Default::default()),
        depth.create_view(&Default::default()),
    );
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
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
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
            .forget_lifetime();
        assert!(slot.paint(&mut pass), "a frame was drawn");
        if let Some(after) = after {
            pass.set_pipeline(after);
            pass.draw(0..3, 0..1);
        }
    }
    let pixels = read_back(device, queue, encoder, &color);
    [pixels[0], pixels[1], pixels[2], pixels[3]]
}

/// A frame of `slot` holding a full-viewport `rgba` triangle at depth 0.5,
/// drawn as the plots draw ([`egui_target`](super::oit::egui_target));
/// whether it was drawn again for `key`.
fn render(
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    slot: &FrameSlot,
    rgba: &str,
    key: u64,
) -> bool {
    let mut resources = eframe::egui_wgpu::CallbackResources::default();
    super::colormap_atlas::prepare(device, queue, &mut resources);
    let Some(atlas) = super::colormap_atlas::bind_group(&resources) else {
        panic!("atlas missing");
    };
    let empty = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[],
    });
    let atlas_layout = super::colormap_atlas::bind_group_layout(device);
    let layouts = [Some(&empty), Some(&atlas_layout)];
    let target = super::oit::egui_target(FORMAT);
    let plot = flat(device, (&layouts, &target), 0.5, rgba, true);
    let mut encoder = device.create_command_encoder(&Default::default());
    let draw = |pass: &mut wgpu::RenderPass<'_>| pass.draw(0..3, 0..1);
    let drew = slot.render(device, &mut encoder, ([SIZE; 2], key), (&plot, atlas), draw);
    queue.submit([encoder.finish()]);
    drew
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn cached_frames_draw_once_and_keep_their_depth() {
    let Some((device, queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED cached_frames_draw_once_and_keep_their_depth: no GPU device");
        return;
    };
    let gpu = (&device, &queue);
    let slot = FrameSlot::new(FORMAT);
    let red = "1.0, 0.0, 0.0, 1.0";
    assert!(render(gpu, &slot, red, 1), "a new frame draws");
    assert!(!render(gpu, &slot, red, 1), "the same key draws nothing");
    assert!(render(gpu, &slot, red, 2), "a new key draws again");

    let opaque = [Some(FORMAT.into())];
    let behind = flat(&device, (&[], &opaque), 0.7, "0.0, 1.0, 0.0, 1.0", false);
    let front = flat(&device, (&[], &opaque), 0.3, "0.0, 0.0, 1.0, 1.0", false);
    let black = wgpu::Color::BLACK;
    assert_eq!(
        paint_then(&device, &queue, &slot, (Some(&behind), black)),
        [255, 0, 0, 255],
        "hidden behind the plot"
    );
    assert_eq!(
        paint_then(&device, &queue, &slot, (Some(&front), black)),
        [0, 0, 255, 255],
        "in front of the plot"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn translucent_frames_blend_as_drawn_straight() {
    let Some((device, queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED translucent_frames_blend_as_drawn_straight: no GPU device");
        return;
    };
    let slot = FrameSlot::new(FORMAT);
    assert!(render((&device, &queue), &slot, "1.0, 0.0, 0.0, 0.5", 1));
    // Half red over white, as straight into the pass: (1, 0.5, 0.5).
    let [r, g, b, _] = paint_then(&device, &queue, &slot, (None, wgpu::Color::WHITE));
    assert_eq!(r, 255);
    for c in [g, b] {
        assert!(
            c.abs_diff(128) <= 1,
            "background shows through by half: {c}"
        );
    }
}
