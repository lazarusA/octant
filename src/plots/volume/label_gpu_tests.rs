//! GPU tests of the categorical label surface.
//! Skipped (with a note) when no adapter is available.

use super::gpu_render::{gpu, params, renderer_wh};
use super::gpu_tests::{N, SIZE, mean_alpha, render};
use super::{VolumeRenderer, VolumeUniformParams};
use crate::utils::colormap::registry;

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

/// The label surface of `data` (`dims` planes) under `p`.
fn draw(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &[f32],
    dims: [usize; 2],
    p: &VolumeUniformParams,
) -> Vec<u8> {
    let renderer: VolumeRenderer = renderer_wh(device, queue, data, dims, false);
    render(device, queue, &renderer, p)
}

#[test]
fn gpu_label_faces_shade_flat() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_label_faces_shade_flat: no GPU adapter, nothing was compared");
        return;
    };
    // One class filling the box: each face facing the camera lit by its own
    // normal (the old mask normals collapsed to one where the class filled
    // the box).
    let one = vec![1.0f32; N * N * N];
    let filled = draw(
        &device,
        &queue,
        &one,
        [N, N],
        &label_params(0.5, 2.5, false, 1.0),
    );
    assert!(mean_alpha(&filled) > 0.05, "the box must show");
    let faces = color_clusters(&filled, 6);
    assert!(
        (2..=3).contains(&faces.len()),
        "{} colors on a box of flat faces",
        faces.len()
    );
}

#[test]
fn gpu_label_boundaries_show_through() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_label_boundaries_show_through: no GPU adapter");
        return;
    };
    // Two touching classes: with translucent surfaces their boundary shows.
    let one = vec![1.0f32; N * N * N];
    let two: Vec<f32> = (0..N * N * N)
        .map(|i| if i % N < N / 2 { 1.0 } else { 2.0 })
        .collect();
    let translucent = label_params(0.5, 2.5, true, 0.4);
    let one_class = mean_alpha(&draw(&device, &queue, &one, [N, N], &translucent));
    let two_classes = mean_alpha(&draw(&device, &queue, &two, [N, N], &translucent));
    assert!(
        two_classes > one_class + 0.005,
        "{two_classes} vs {one_class}"
    );
}

#[test]
fn gpu_label_thin_class_is_never_missed() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_label_thin_class_is_never_missed: no GPU adapter");
        return;
    };
    // Out of the color range is background; a one-voxel class among 600
    // planes is never stepped over.
    const W: usize = 16;
    const D: usize = 600;
    let mut plane = vec![0.0f32; W * W * D];
    plane[W * W * 301..W * W * 302].fill(1.0);
    let opaque = label_params(0.5, 2.5, false, 1.0);
    let thin = draw(&device, &queue, &plane, [W, W], &opaque);
    let full = draw(&device, &queue, &vec![1.0; W * W * D], [W, W], &opaque);
    let center = (SIZE / 2 - 6)..(SIZE / 2 + 6);
    let missed = center.clone().any(|y| {
        center
            .clone()
            .any(|x| thin[((y * SIZE + x) * 4 + 3) as usize] != 255)
    });
    assert!(!missed, "a ray missed the class");
    assert!(
        mean_alpha(&thin) < mean_alpha(&full),
        "background must not draw"
    );
}
