//! The bake writes a target buffer with the CPU blend's packed colors.

use super::channels_tests::{block, configs, cpu};
use crate::data::slicing::CompositeStretch;

/// Reads `buffer` (`cells` `f32`s) back from the GPU.
fn read_back(device: &wgpu::Device, queue: &wgpu::Queue, buffer: &wgpu::Buffer) -> Vec<f32> {
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: buffer.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, buffer.size());
    queue.submit([encoder.finish()]);
    staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let bytes = staging.slice(..).get_mapped_range().expect("mapped");
    bytemuck::cast_slice(&bytes).to_vec()
}

/// Every cell baked as the CPU packs it (within one level of rounding per
/// color), NaN where no channel is valid. Skipped without a GPU.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn bakes_what_the_cpu_blends() {
    use crate::data::slicing::composite::utils::unpack_rgb;
    let Some((device, queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED bakes_what_the_cpu_blends: no GPU device");
        return;
    };
    let (block, configs) = (block(), configs());
    let mut stretch = CompositeStretch::default();
    let (packed, planes) = cpu(&block, &configs, &mut stretch);
    let renderer = super::ChannelRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, 3, 64);
    for (i, plane) in planes.iter().enumerate() {
        renderer.write_plane(&queue, i, plane);
    }
    let params = super::ChannelParams::of(&configs, &stretch, (0, 3), 64, None);
    renderer.update_params(&queue, &params);
    let target = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 64 * 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    renderer.bake(&device, &mut encoder, &params, &target);
    queue.submit([encoder.finish()]);
    for (cell, (got, want)) in read_back(&device, &queue, &target)
        .iter()
        .zip(&packed)
        .enumerate()
    {
        if want.is_nan() {
            assert!(got.is_nan(), "cell {cell}: baked {got}, CPU NaN");
            continue;
        }
        let (got, want) = (unpack_rgb(*got), unpack_rgb(*want));
        let close = [(got.0, want.0), (got.1, want.1), (got.2, want.2)]
            .iter()
            .all(|(g, w)| (g - w).abs() <= 1.0);
        assert!(close, "cell {cell}: baked {got:?}, CPU {want:?}");
    }
}
