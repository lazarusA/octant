//! The volume bake writes the raymarcher's textures as `encode_rgba`
//! encodes the CPU blend.

use super::VolumeTargets;
use super::volume::encode::{Dims, encode_rgba};
use crate::data::slicing::{ChannelColorConfig, CompositeStretch};

const W: usize = 5;
const H: usize = 4;
const D: usize = 3;

/// Three channels of a `W x H x D` volume with missing voxels, one voxel
/// missing in every channel (filled from its neighbors).
fn channels() -> Vec<Vec<f32>> {
    (0..3)
        .map(|c| {
            (0..W * H * D)
                .map(|i| match (i, (i + c) % 5) {
                    (7, _) => f32::NAN,
                    (_, 0) => f32::NAN,
                    _ => ((i * 37 + c * 11) % 200) as f32,
                })
                .collect()
        })
        .collect()
}

fn texture(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
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
        usage: usage | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// `texture`'s texels (`texel` bytes each), row padding dropped.
fn read_back(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    texel: usize,
) -> Vec<u8> {
    let row = (W * texel).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (row * H * D) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row as u32),
                rows_per_image: Some(H as u32),
            },
        },
        texture.size(),
    );
    queue.submit([encoder.finish()]);
    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let bytes = buffer.slice(..).get_mapped_range().expect("mapped");
    bytes
        .chunks(row)
        .flat_map(|r| r[..W * texel].to_vec())
        .collect()
}

/// Every texel within one level of the CPU encoding, validity exact.
/// Skipped without a GPU.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn bakes_volumes_as_the_cpu_encodes_them() {
    let Some((device, queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED bakes_volumes_as_the_cpu_encodes_them: no GPU device");
        return;
    };
    let configs: Vec<_> = (0..3)
        .map(|i| {
            ChannelColorConfig::new(
                i,
                format!("c{i}"),
                [[255, 40, 0], [0, 200, 90], [60, 0, 255]][i],
            )
        })
        .collect();
    let planes = channels();
    let mut stretch = CompositeStretch::default();
    for (c, p) in planes.iter().enumerate() {
        stretch.channel(c, p, None, 1.0);
    }
    let packed: Vec<f32> = (0..W * H * D)
        .map(|i| {
            crate::data::slicing::blend_samples(&configs, &stretch, (0, 3), |c| {
                planes[c].get(i).copied()
            })
        })
        .collect();
    let want = encode_rgba(&packed, Dims { w: W, h: H, d: D }, 0..D);

    let renderer =
        super::ChannelRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm, 3, W * H * D);
    for (c, p) in planes.iter().enumerate() {
        renderer.write_plane(&queue, c, p);
    }
    let params = super::ChannelParams::of(&configs, &stretch, (0, 3), (W * H * D) as u32, None);
    renderer.update_params(&queue, &params);
    let colors = texture(
        &device,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::STORAGE_BINDING,
    );
    let validity = texture(
        &device,
        wgpu::TextureFormat::R8Unorm,
        wgpu::TextureUsages::TEXTURE_BINDING,
    );
    let targets = VolumeTargets {
        colors: &colors,
        validity: &validity,
        dims: [W, H, D].map(|n| n as u32),
    };
    let mut encoder = device.create_command_encoder(&Default::default());
    assert!(renderer.bake_volume(&device, &mut encoder, &params, &targets));
    queue.submit([encoder.finish()]);

    assert_eq!(
        read_back(&device, &queue, &validity, 1),
        want.validity,
        "validity"
    );
    let texels = read_back(&device, &queue, &colors, 4);
    for (i, (got, want)) in texels.chunks(4).zip(&want.texels).enumerate() {
        let close = got.iter().zip(want).all(|(g, w)| g.abs_diff(*w) <= 1);
        assert!(close, "voxel {i}: baked {got:?}, CPU {want:?}");
    }
    // Nothing changed: no second bake.
    let mut encoder = device.create_command_encoder(&Default::default());
    assert!(!renderer.bake_volume(&device, &mut encoder, &params, &targets));
}
