//! GPU tests of DVR's brightness and its value surface.
//! Skipped (with a note) when no adapter is available.

use super::VolumeUniformParams;
use super::gpu_render::{gpu, params};
use super::gpu_tests::{SIZE, blob, mean_alpha, render, renderer};
use crate::utils::colormap::registry;

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
