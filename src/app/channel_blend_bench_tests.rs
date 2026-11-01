//! What blending a large multi-channel composite costs, CPU against GPU,
//! timed by hand: `cargo test --release --lib channel_blend_timed --
//! --ignored --nocapture`. Four channels of 3515 x 7031 cells, the size of
//! TESSERA dclimate's coarsest RGB level read over the whole globe.

use std::time::{Duration, Instant};

use crate::data::slicing::{ChannelColorConfig, CompositeStretch};

const W: usize = 7031;
const H: usize = 3515;
const CHANNELS: usize = 4;
const RUNS: usize = 5;

/// A device with the adapter's own limits, as the app requests them.
fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(async {
        let adapter = wgpu::Instance::default()
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok()?;
        let descriptor = wgpu::DeviceDescriptor {
            required_limits: adapter.limits(),
            ..Default::default()
        };
        adapter.request_device(&descriptor).await.ok()
    })
}

fn block() -> crate::data::octant_block::OctantBlock {
    let values: Vec<f32> = (0..CHANNELS * W * H)
        .map(|i| ((i.wrapping_mul(2_654_435_761)) % 256) as f32)
        .collect();
    crate::data::octant_block::OctantBlock::new(
        "rgb".into(),
        vec![CHANNELS, H, W],
        ["band", "lat", "lon"].map(String::from).to_vec(),
        vec![0; 3],
        values,
        Default::default(),
        std::collections::HashMap::new(),
    )
}

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

/// Draws `renderer` over `heatmap` into a 1920 x 1080 target and waits.
fn frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    heatmap: &crate::plots::HeatmapRenderer,
    renderer: &crate::plots::ChannelRenderer,
) {
    let size = wgpu::Extent3d {
        width: 1920,
        height: 1080,
        depth_or_array_layers: 1,
    };
    let texture = |format| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    };
    let target = texture(wgpu::TextureFormat::Rgba8Unorm).create_view(&Default::default());
    let depth = texture(wgpu::TextureFormat::Depth32Float).create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut rpass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations::default(),
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: None,
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
            .forget_lifetime();
        renderer.draw(&mut rpass, heatmap);
    }
    queue.submit([encoder.finish()]);
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
}

#[test]
#[ignore = "benchmark"]
fn channel_blend_timed() {
    let Some((device, queue)) = device() else {
        eprintln!("SKIPPED channel_blend_timed: no GPU device");
        return;
    };
    // As `OctantApp::new` records them.
    crate::plots::limits::set_from_device(&device.limits());
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let block = block();
    let mut configs: Vec<_> = (0..CHANNELS)
        .map(|i| ChannelColorConfig::new(i, format!("b{i}"), [255, (i * 60) as u8, 0]))
        .collect();
    let blend = |configs: &[ChannelColorConfig], stretch: &mut CompositeStretch| {
        crate::data::slicing::slice_multichannel_composite_nd(
            &block,
            0,
            2,
            1,
            (0, W),
            (0, H),
            &[0, 0, 0],
            configs,
            1,
            stretch,
        )
        .expect("blend")
    };
    let mut stretch = CompositeStretch::default();
    let packed = blend(&configs, &mut stretch);
    let heatmap = crate::plots::HeatmapRenderer::new(&device, format, &packed.values, W, H);
    let wait = || {
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
    };
    assert!(
        crate::plots::ChannelRenderer::fits(CHANNELS, W * H),
        "planes fit"
    );
    let renderer = crate::plots::ChannelRenderer::new(&device, format, CHANNELS, W * H);

    let (mut cpu_change, mut gpu_change, mut cpu_block, mut gpu_block, mut draw) =
        (vec![], vec![], vec![], vec![], vec![]);
    for run in 0..RUNS {
        configs[1].color_rgb = [0, (run * 40) as u8, 255];
        // Before: every display change re-blends on the CPU and uploads.
        let start = Instant::now();
        let packed = blend(&configs, &mut stretch);
        heatmap.update_data(&queue, &packed.values);
        wait();
        cpu_change.push(start.elapsed());
        // After: the uniforms change, and the next frame blends.
        let start = Instant::now();
        let params = crate::plots::ChannelParams::of(
            &configs,
            &stretch,
            (0, CHANNELS),
            (W * H) as u32,
            None,
        );
        renderer.update_params(&queue, &params);
        frame(&device, &queue, &heatmap, &renderer);
        gpu_change.push(start.elapsed());
        // A block arriving: blend and upload, against planes cut and uploaded.
        let start = Instant::now();
        let mut fresh = CompositeStretch::default();
        let packed = blend(&configs, &mut fresh);
        heatmap.update_data(&queue, &packed.values);
        wait();
        cpu_block.push(start.elapsed());
        let start = Instant::now();
        let window = crate::app::block_loading::projection_2d::PlaneWindow {
            x_dim: 2,
            y_dim: 1,
            x_range: (0, W),
            y_range: (0, H),
            fixed: &[0, 0, 0],
        };
        let mut fresh = CompositeStretch::default();
        crate::app::block_loading::projection_channels::each_plane(&block, 0, &window, |c, p| {
            fresh.channel(c, &p.values, None, 1.0);
            renderer.write_plane(&queue, c, &p.values);
        })
        .expect("planes");
        wait();
        gpu_block.push(start.elapsed());
        // Every frame the GPU path draws, once the upload has settled.
        frame(&device, &queue, &heatmap, &renderer);
        let start = Instant::now();
        frame(&device, &queue, &heatmap, &renderer);
        draw.push(start.elapsed());
    }
    eprintln!("{CHANNELS} channels of {W} x {H} (median of {RUNS} runs)");
    eprintln!(
        "display change, CPU blend + upload   {:>10.2?}",
        median(cpu_change)
    );
    eprintln!(
        "display change, GPU uniforms + frame {:>10.2?}",
        median(gpu_change)
    );
    eprintln!(
        "block arrives, CPU blend + upload    {:>10.2?}",
        median(cpu_block)
    );
    eprintln!(
        "block arrives, GPU planes upload     {:>10.2?}",
        median(gpu_block)
    );
    eprintln!(
        "one more 1920 x 1080 frame (GPU)     {:>10.2?}",
        median(draw)
    );
}
