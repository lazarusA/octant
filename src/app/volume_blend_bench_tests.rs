//! What a display change of a multi-channel composite volume costs, CPU
//! blend and encoding against the GPU bake, timed by hand: `cargo test
//! --release --lib volume_blend_timed -- --ignored --nocapture`. Four
//! channels of 256 x 256 x 128 voxels, the most a volume's channels hold
//! (`MAX_GPU_STORAGE_BUFFER_BYTES`).

use std::time::Instant;

use super::channel_blend_bench_tests::{RUNS, device, median};
use crate::data::slicing::{ChannelColorConfig, CompositeStretch};
use crate::plots::volume::encode::{Dims, encode_rgba};

const W: usize = 256;
const H: usize = 256;
const D: usize = 128;
const C: usize = 4;

fn texture(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::Texture {
    let storage = match format {
        wgpu::TextureFormat::Rgba8Unorm => wgpu::TextureUsages::STORAGE_BINDING,
        _ => wgpu::TextureUsages::empty(),
    };
    device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: W as u32,
            height: H as u32,
            depth_or_array_layers: D as u32,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | storage,
        view_formats: &[],
    })
}

#[test]
#[ignore = "benchmark"]
fn volume_blend_timed() {
    let Some((device, queue)) = device() else {
        eprintln!("SKIPPED volume_blend_timed: no GPU device");
        return;
    };
    let n = W * H * D;
    let planes: Vec<Vec<f32>> = (0..C)
        .map(|c| (0..n).map(|i| ((i * 31 + c * 7) % 251) as f32).collect())
        .collect();
    let mut configs: Vec<_> = (0..C)
        .map(|i| ChannelColorConfig::new(i, format!("c{i}"), [255, (i * 60) as u8, 0]))
        .collect();
    let mut stretch = CompositeStretch::default();
    for (c, p) in planes.iter().enumerate() {
        stretch.channel(c, p, None, 1.0);
    }
    let colors = texture(&device, wgpu::TextureFormat::Rgba8Unorm);
    let validity = texture(&device, wgpu::TextureFormat::R8Unorm);
    let renderer =
        crate::plots::ChannelRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, C, n);
    for (c, p) in planes.iter().enumerate() {
        renderer.write_plane(&queue, c, p);
    }
    let targets = crate::plots::VolumeTargets {
        id: crate::plots::BakeId::next(),
        colors: &colors,
        validity: &validity,
        dims: [W, H, D].map(|v| v as u32),
    };
    let wait = || {
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
    };
    let (mut cpu, mut gpu) = (vec![], vec![]);
    for run in 0..RUNS {
        configs[1].color_rgb = [0, (run * 40) as u8, 255];
        // Before: blend every voxel, encode the texels, upload them.
        let start = Instant::now();
        let packed: Vec<f32> = (0..n)
            .map(|i| {
                crate::data::slicing::blend_samples(&configs, &stretch, (0, C), |c| {
                    planes[c].get(i).copied()
                })
            })
            .collect();
        let encoded = encode_rgba(&packed, Dims { w: W, h: H, d: D }, 0..D);
        queue.write_texture(
            colors.as_image_copy(),
            bytemuck::cast_slice(&encoded.texels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((W * 4) as u32),
                rows_per_image: Some(H as u32),
            },
            colors.size(),
        );
        queue.submit([]);
        wait();
        cpu.push(start.elapsed());
        // After: new uniforms and one bake.
        let start = Instant::now();
        let params = crate::plots::ChannelParams::of(&configs, &stretch, (0, C), n as u32, None);
        renderer.update_params(&queue, &params);
        let mut encoder = device.create_command_encoder(&Default::default());
        renderer.bake_volume(&device, &mut encoder, &params, &targets);
        queue.submit([encoder.finish()]);
        wait();
        gpu.push(start.elapsed());
    }
    eprintln!("volume, {C} channels of {W} x {H} x {D} (median of {RUNS} runs)");
    eprintln!(
        "display change, CPU blend + encode + upload {:>10.2?}",
        median(cpu)
    );
    eprintln!(
        "display change, GPU bake                   {:>10.2?}",
        median(gpu)
    );
}
