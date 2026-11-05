//! GPU tests: renders small volumes offscreen and compares the images.
//! Skipped (with a note) when no adapter is available.

use super::gpu_render::{atlas, camera, gpu, params, render_sized, renderer_n, renderer_wh};
use super::{VolumeRenderer, VolumeUniformParams};
use crate::utils::colormap::registry;

const SIZE: u32 = 64;
const N: usize = 16;

/// Smooth blob in [0, 1] with the corner octant missing.
fn blob() -> Vec<f32> {
    let mut values = Vec::with_capacity(N * N * N);
    for z in 0..N {
        for y in 0..N {
            for x in 0..N {
                let p = [x, y, z].map(|c| (c as f32 + 0.5) / N as f32 - 0.5);
                let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
                let missing = x < N / 4 && y < N / 4 && z < N / 4;
                values.push(if missing {
                    f32::NAN
                } else {
                    (1.0 - 2.0 * r).clamp(0.0, 1.0)
                });
            }
        }
    }
    values
}

fn render(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &VolumeRenderer,
    p: &VolumeUniformParams,
) -> Vec<u8> {
    render_sized(device, queue, renderer, p, SIZE)
}

fn renderer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &[f32],
    hardware: bool,
) -> VolumeRenderer {
    renderer_n(device, queue, data, N, hardware)
}

fn max_diff(a: &[u8], b: &[u8]) -> u8 {
    a.iter()
        .zip(b)
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

fn mean_alpha(pixels: &[u8]) -> f32 {
    let sum: u32 = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| u32::from(p[3]))
        .sum();
    sum as f32 / (SIZE * SIZE) as f32 / 255.0
}

#[test]
fn gpu_volume_rendering() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED gpu_volume_rendering: no GPU adapter, nothing was compared");
        return;
    };
    let data = blob();
    let manual = renderer(&device, &queue, &data, false);
    let dvr = render(&device, &queue, &manual, &params(0, 1.0, 0));
    assert_eq!(dvr.len(), (SIZE * SIZE * 4) as usize, "readback failed");
    assert!(
        mean_alpha(&dvr) > 0.01,
        "the blob must be visible: {}",
        mean_alpha(&dvr)
    );
    // Dithered output stays premultiplied (no channel above alpha).
    assert!(
        dvr.as_chunks::<4>()
            .0
            .iter()
            .all(|p| p[..3].iter().all(|&c| c <= p[3]))
    );

    // Lighting shades color only: opacity is unchanged.
    let unlit_params = VolumeUniformParams {
        lighting: false,
        ..params(0, 1.0, 0)
    };
    let unlit = render(&device, &queue, &manual, &unlit_params);
    assert!(
        dvr.iter()
            .skip(3)
            .step_by(4)
            .eq(unlit.iter().skip(3).step_by(4))
    );
    assert_ne!(dvr, unlit, "lighting must change the shading");

    // Opacity is corrected for the step: quality changes grain, not density.
    let fine = render(&device, &queue, &manual, &params(0, 2.0, 0));
    let coarse = render(&device, &queue, &manual, &params(0, 0.5, 0));
    let (a_fine, a_coarse) = (mean_alpha(&fine), mean_alpha(&coarse));
    assert!(
        (a_fine - a_coarse).abs() < 0.02,
        "alpha {a_fine} vs {a_coarse}"
    );

    // A ring buffer rotated by `shift` renders as the unrotated volume.
    let shift = 5;
    let mut rolled = vec![0.0; data.len()];
    for (i, v) in data.iter().enumerate() {
        let (x, row) = (i % N, i / N);
        rolled[row * N + (x + shift) % N] = *v;
    }
    let rolled_renderer = renderer(&device, &queue, &rolled, false);
    for algorithm in [0, 1] {
        let base = render(&device, &queue, &manual, &params(algorithm, 1.0, 0));
        let ring = render(
            &device,
            &queue,
            &rolled_renderer,
            &params(algorithm, 1.0, shift as u32),
        );
        assert!(
            max_diff(&base, &ring) <= 1,
            "mode {algorithm} differs across the seam"
        );
    }

    // Hardware filtering matches the eight-tap manual filter.
    if filterable {
        let hardware = renderer(&device, &queue, &data, true);
        for algorithm in [0, 1, 3] {
            let a = render(&device, &queue, &manual, &params(algorithm, 1.0, 0));
            let b = render(&device, &queue, &hardware, &params(algorithm, 1.0, 0));
            assert!(
                max_diff(&a, &b) <= 6,
                "mode {algorithm}: hardware vs manual {}",
                max_diff(&a, &b)
            );
        }
    } else {
        eprintln!("SKIPPED hardware filter comparison: no FLOAT32_FILTERABLE");
    }

    // A volume with no data draws nothing.
    let empty = renderer(&device, &queue, &vec![f32::NAN; data.len()], false);
    assert!(empty.textures.has_invalid());
    assert_eq!(
        mean_alpha(&render(&device, &queue, &empty, &params(0, 1.0, 0))),
        0.0
    );
}

#[test]
fn gpu_frame_cache_rerenders_only_on_change() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_frame_cache_rerenders_only_on_change: no GPU adapter");
        return;
    };
    let data = blob();
    let renderer = renderer(&device, &queue, &data, false);
    let resources = atlas(&device, &queue);
    let mut encoder = device.create_command_encoder(&Default::default());
    let mut frame = |p: &VolumeUniformParams, size: [u32; 2]| {
        renderer.render_frame(&device, &queue, &mut encoder, p, size, &resources)
    };
    let base = params(0, 1.0, 0);
    assert!(frame(&base, [32, 32]), "first frame renders");
    assert!(!frame(&base, [32, 32]), "unchanged view reuses the frame");
    let panned = VolumeUniformParams {
        camera: camera(0.6, [0.1, 0.0, 0.0]),
        ..base
    };
    assert!(frame(&panned, [32, 32]), "pan re-renders");
    let turned = VolumeUniformParams {
        camera: camera(1.0, [0.0; 3]),
        ..base
    };
    assert!(frame(&turned, [32, 32]), "camera change re-renders");
    assert!(frame(&turned, [16, 16]), "size change re-renders");
    renderer.update_data(&queue, &data);
    let mut encoder = device.create_command_encoder(&Default::default());
    assert!(
        renderer.render_frame(&device, &queue, &mut encoder, &turned, [16, 16], &resources),
        "new data re-renders"
    );
}

/// Mean of the premultiplied color channels over the frame.
fn mean_color(pixels: &[u8]) -> f32 {
    let sum: u32 = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]))
        .sum();
    sum as f32 / (3 * SIZE * SIZE) as f32 / 255.0
}

#[test]
fn gpu_lighting_does_not_dim() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_lighting_does_not_dim: no GPU adapter, nothing was compared");
        return;
    };
    let p = params(0, 1.0, 0);
    // Lighting shades, it does not dim the volume.
    let blob = renderer(&device, &queue, &blob(), false);
    let lit = mean_color(&render(&device, &queue, &blob, &p));
    let unlit_params = VolumeUniformParams {
        lighting: false,
        ..p
    };
    let unlit = mean_color(&render(&device, &queue, &blob, &unlit_params));
    assert!(lit >= 0.9 * unlit, "lit {lit} vs unlit {unlit}");
}

/// Alpha channel of a frame.
fn alphas(pixels: &[u8]) -> Vec<u8> {
    pixels.iter().skip(3).step_by(4).copied().collect()
}

#[test]
fn gpu_value_surface() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_value_surface: no GPU adapter, nothing was compared");
        return;
    };
    let blob = renderer(&device, &queue, &blob(), false);
    let opaque = VolumeUniformParams {
        transparency: false,
        ..params(0, 1.0, 0)
    };
    // The surface at 0.5 is the one the range [0.5, 1] draws (the blob's
    // values stay within 1)...
    let mut range = opaque;
    range.color.cmin = 0.5;
    let value = VolumeUniformParams {
        surface: 1,
        isovalue: 0.5,
        ..opaque
    };
    let by_range = alphas(&render(&device, &queue, &blob, &range));
    let by_value = alphas(&render(&device, &queue, &blob, &value));
    assert!(by_value.iter().any(|&a| a > 0), "the surface must show");
    assert_eq!(by_range, by_value, "same surface, same coverage");
    // ...whatever the color range, which only colors it.
    let mut recolored = value;
    recolored.color.cmax = 0.3;
    assert_eq!(
        by_value,
        alphas(&render(&device, &queue, &blob, &recolored))
    );
    // A higher value draws a smaller surface.
    let inner = VolumeUniformParams {
        isovalue: 0.8,
        ..value
    };
    let inner = mean_alpha(&render(&device, &queue, &blob, &inner));
    assert!(inner < 0.5 * mean_alpha(&render(&device, &queue, &blob, &value)));
}

#[test]
fn gpu_mip_never_skips_a_voxel() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_mip_never_skips_a_voxel: no GPU adapter, nothing was compared");
        return;
    };
    // Deeper than the 384 samples rays took: a one-voxel plane among 600
    // must show on every ray, as a volume full of its value does.
    const W: usize = 16;
    const D: usize = 600;
    let mut plane = vec![0.0f32; W * W * D];
    plane[W * W * 301..W * W * 302].fill(1.0);
    let full = vec![1.0f32; W * W * D];
    let p = params(1, 1.0, 0);
    let draw = |data: &[f32]| {
        let r = renderer_wh(&device, &queue, data, [W, W], false);
        render(&device, &queue, &r, &p)
    };
    let (plane, full) = (draw(&plane), draw(&full));
    assert!(mean_alpha(&full) > 0.05, "the box must show");
    // Rays through the frame's center cross the box's middle plane; those
    // near its edges may enter after or leave before it.
    let center = |pixels: &[u8]| -> Vec<u8> {
        let rows = (SIZE / 2 - 6)..(SIZE / 2 + 6);
        rows.flat_map(|y| {
            let start = ((y * SIZE + SIZE / 2 - 6) * 4) as usize;
            pixels[start..start + 48].to_vec()
        })
        .collect()
    };
    assert!(
        max_diff(&center(&plane), &center(&full)) <= 2,
        "a ray stepped over the plane"
    );
}

/// Label surface parameters over classes in `cmin..=cmax`.
fn label_params(cmin: f32, cmax: f32, transparency: bool, opacity: f32) -> VolumeUniformParams {
    let mut p = params(4, 1.0, 0);
    p.transparency = transparency;
    p.color.cmin = cmin;
    p.color.cmax = cmax;
    p.color.opacity = opacity;
    p
}

/// Colors of the opaque pixels, merged within `tolerance` (the dither).
fn color_clusters(pixels: &[u8], tolerance: u8) -> Vec<[u8; 3]> {
    let mut clusters: Vec<[u8; 3]> = vec![];
    for p in pixels.as_chunks::<4>().0.iter().filter(|p| p[3] == 255) {
        let c = [p[0], p[1], p[2]];
        let near = |k: &[u8; 3]| (0..3).all(|i| k[i].abs_diff(c[i]) <= tolerance);
        if !clusters.iter().any(near) {
            clusters.push(c);
        }
    }
    clusters
}

#[test]
fn gpu_label_surfaces() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_label_surfaces: no GPU adapter, nothing was compared");
        return;
    };
    let draw = |data: &[f32], dims: [usize; 2], p: &VolumeUniformParams| {
        render(
            &device,
            &queue,
            &renderer_wh(&device, &queue, data, dims, false),
            p,
        )
    };

    // One class filling the box: its faces shade flat, one color per face.
    let one = vec![1.0f32; N * N * N];
    let opaque = label_params(0.5, 2.5, false, 1.0);
    let filled = draw(&one, [N, N], &opaque);
    assert!(mean_alpha(&filled) > 0.05, "the box must show");
    let faces = color_clusters(&filled, 6);
    // Each face facing the camera lit by its own normal (the old mask
    // normals collapsed to one where the class filled the box).
    assert!(
        (2..=3).contains(&faces.len()),
        "{} colors on a box of flat faces",
        faces.len()
    );

    // Two touching classes: with translucent surfaces their boundary shows.
    let two: Vec<f32> = (0..N * N * N)
        .map(|i| if i % N < N / 2 { 1.0 } else { 2.0 })
        .collect();
    let translucent = label_params(0.5, 2.5, true, 0.4);
    let one_class = mean_alpha(&draw(&one, [N, N], &translucent));
    let two_classes = mean_alpha(&draw(&two, [N, N], &translucent));
    assert!(
        two_classes > one_class + 0.005,
        "{two_classes} vs {one_class}"
    );

    // Out of the color range is background; a one-voxel class among 600
    // planes is never stepped over.
    const W: usize = 16;
    const D: usize = 600;
    let mut plane = vec![0.0f32; W * W * D];
    plane[W * W * 301..W * W * 302].fill(1.0);
    let thin = draw(&plane, [W, W], &opaque);
    let center = (SIZE / 2 - 6)..(SIZE / 2 + 6);
    let missed = center.clone().any(|y| {
        center
            .clone()
            .any(|x| thin[((y * SIZE + x) * 4 + 3) as usize] != 255)
    });
    assert!(!missed, "a ray missed the class");
    assert!(
        mean_alpha(&thin) < mean_alpha(&filled),
        "background must not draw"
    );
}
