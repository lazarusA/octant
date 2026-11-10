//! The bake writes a target buffer with the CPU blend's packed colors.

use super::Thinning;
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
    let id = super::BakeId::next();
    let mut encoder = device.create_command_encoder(&Default::default());
    let bake = |encoder: &mut wgpu::CommandEncoder, id| {
        renderer.bake(&device, encoder, &params, (id, &target, Thinning::NONE))
    };
    assert!(bake(&mut encoder, id), "a new target bakes");
    assert!(!bake(&mut encoder, id), "a baked one is skipped");
    // A rebuilt mesh (new renderer, new id) always bakes again.
    assert!(bake(&mut encoder, super::BakeId::next()), "a new id bakes");
    queue.submit([encoder.finish()]);
    assert_close(&read_back(&device, &queue, &target), &packed);

    // A grid thinned by 2: 4 x 4 cells, each the middle of its 2 x 2.
    let small = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16 * 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let thinning = Thinning {
        stride: 2,
        width: 4,
        height: 4,
        full_width: 8,
        full_height: 8,
    };
    let mut encoder = device.create_command_encoder(&Default::default());
    renderer.bake(
        &device,
        &mut encoder,
        &params,
        (super::BakeId::next(), &small, thinning),
    );
    queue.submit([encoder.finish()]);
    let want: Vec<f32> = (0..16)
        .map(|i| packed[(i / 4 * 2 + 1) * 8 + i % 4 * 2 + 1])
        .collect();
    assert_close(&read_back(&device, &queue, &small), &want);

    // The same cells as two 8 x 4 planes, thinned by 2 along x and y only
    // (a point cloud): 4 x 2 cells a plane, every plane.
    let thinning = Thinning {
        stride: 2,
        width: 4,
        height: 2,
        full_width: 8,
        full_height: 4,
    };
    let mut encoder = device.create_command_encoder(&Default::default());
    renderer.bake(
        &device,
        &mut encoder,
        &params,
        (super::BakeId::next(), &small, thinning),
    );
    queue.submit([encoder.finish()]);
    let want: Vec<f32> = (0..16)
        .map(|i| {
            let (z, y, x) = (i / 8, i % 8 / 4, i % 4);
            packed[(z * 4 + y * 2 + 1) * 8 + x * 2 + 1]
        })
        .collect();
    assert_close(&read_back(&device, &queue, &small), &want);

    // A 7 x 9 grid thinned by 3, not a whole multiple: 3 x 3 cells, each
    // the cell under its middle, as `sample_index` picks it.
    let thinning = Thinning {
        stride: 3,
        width: 3,
        height: 3,
        full_width: 7,
        full_height: 9,
    };
    let mut encoder = device.create_command_encoder(&Default::default());
    renderer.bake(
        &device,
        &mut encoder,
        &params,
        (super::BakeId::next(), &small, thinning),
    );
    queue.submit([encoder.finish()]);
    use crate::data::render::decimate::sample_index;
    let want: Vec<f32> = (0..9)
        .map(|i| packed[sample_index(i / 3, 3, 9) * 7 + sample_index(i % 3, 3, 7)])
        .collect();
    assert_close(&read_back(&device, &queue, &small)[..9], &want);
}

/// Each baked cell as the CPU packs it, within one level per color.
fn assert_close(baked: &[f32], packed: &[f32]) {
    use crate::data::slicing::composite::utils::unpack_rgb;
    for (cell, (got, want)) in baked.iter().zip(packed).enumerate() {
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

/// A raw upload into a baked mesh or point cloud (`update_data`,
/// `update_data_range`) overwrites its composite, so the next frame bakes it
/// again. Skipped without a GPU.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_raw_upload_bakes_the_mesh_again() {
    let Some((device, queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED a_raw_upload_bakes_the_mesh_again: no GPU device");
        return;
    };
    let (block, configs) = (block(), configs());
    let mut stretch = CompositeStretch::default();
    let (_, planes) = cpu(&block, &configs, &mut stretch);
    let renderer = super::ChannelRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, 3, 64);
    for (i, plane) in planes.iter().enumerate() {
        renderer.write_plane(&queue, i, plane);
    }
    let params = super::ChannelParams::of(&configs, &stretch, (0, 3), 64, None);
    renderer.update_params(&queue, &params);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mesh = super::SurfaceRenderer::new_surface(&device, format, &[0.0; 64], 8, 8);
    let mut encoder = device.create_command_encoder(&Default::default());
    let points = super::PointCloudRenderer::new(&device, format, &[0.0; 64], 8, 8);
    let mut bake =
        |id, buffer| renderer.bake(&device, &mut encoder, &params, (id, buffer, Thinning::NONE));
    assert!(
        bake(mesh.bake_id.id(), &mesh.data_buffer),
        "a new mesh bakes"
    );
    assert!(
        !bake(mesh.bake_id.id(), &mesh.data_buffer),
        "a baked one is skipped"
    );
    mesh.update_data(&queue, &[0.0; 64]);
    assert!(
        bake(mesh.bake_id.id(), &mesh.data_buffer),
        "raw values replaced it"
    );
    assert!(
        bake(points.bake_id.id(), &points.data_buffer),
        "new points bake"
    );
    points.update_data_range(&queue, 0, &[0.0; 8]);
    assert!(
        bake(points.bake_id.id(), &points.data_buffer),
        "a raw slab replaced it"
    );
}
