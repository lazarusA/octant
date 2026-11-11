//! An overlay mesh draped on the base (`Mesh3DCallback::heights`) takes the
//! base's relief on a GPU: it covers exactly the base's pixels, which its
//! own flat heights would not. Skipped (with a note) without a GPU.

use std::sync::Arc;

use super::camera::OrbitCamera;
use super::common::{Mesh3DUniformParams, PlotColorParams};
use super::slice::SliceUniform;
use super::test_frame::{FORMAT, pixels, rect};
use super::{Mesh3DCallback, Mesh3DRenderer};

/// `renderer` drawn in `mode`, its heights from `heights` when given.
fn draw(
    renderer: &Arc<Mesh3DRenderer>,
    (sphere, mode): (bool, u32),
    heights: Option<&Arc<Mesh3DRenderer>>,
) -> Mesh3DCallback {
    let color = PlotColorParams {
        cmax: 100.0,
        ..Default::default()
    };
    let camera = OrbitCamera {
        rotation_x: 0.6,
        ..OrbitCamera::default()
    };
    Mesh3DCallback {
        renderer: renderer.clone(),
        params: Mesh3DUniformParams {
            slice: SliceUniform::OFF,
            color,
            camera: camera.uniform(),
            displacement_strength: 1.0,
            mode,
            coord_mode: 0,
            lon_bounds: [-std::f32::consts::PI, std::f32::consts::PI],
            lat_bounds: [-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2],
            flip_height: false,
            height_range: [color.cmin, color.cmax],
            height_composite: false,
        },
        cube_mode_idx: if sphere { 3 } else { 2 },
        rect: rect(),
        transparency: Default::default(),
        channels: None,
        thinning: super::Thinning::NONE,
        heights: heights.cloned(),
    }
}

/// Which pixels a frame covers.
fn mask(pixels: &[u8]) -> Vec<bool> {
    pixels.as_chunks::<4>().0.iter().map(|p| p[3] > 0).collect()
}

#[test]
fn gpu_overlays_drape_on_the_base_relief() {
    let Some((device, queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED gpu_overlays_drape_on_the_base_relief: no GPU device");
        return;
    };
    let (w, h) = (16, 8);
    // The base at the top of its range, the overlay at the bottom (flat).
    let base_values = [100.0; 16 * 8];
    let overlay_values = [0.0; 16 * 8];
    let new = |sphere, values: &[f32]| {
        Arc::new(match sphere {
            true => Mesh3DRenderer::new_sphere(&device, FORMAT, values, w, h),
            false => Mesh3DRenderer::new_surface(&device, FORMAT, values, w, h),
        })
    };
    // Surface bumpy, steps, blocks; sphere bumpy, steps, blocks.
    let cases = [
        (false, 0),
        (false, 1),
        (false, 2),
        (true, 1),
        (true, 2),
        (true, 3),
    ];
    for (sphere, mode) in cases {
        let (base, overlay) = (new(sphere, &base_values), new(sphere, &overlay_values));
        let frame = |r, heights| mask(&pixels(&device, &queue, &draw(r, (sphere, mode), heights)));
        let relief = frame(&base, None);
        let what = format!("sphere {sphere} mode {mode}");
        assert!(relief.iter().any(|&p| p), "{what}: the base draws");
        assert_eq!(frame(&overlay, Some(&base)), relief, "{what}: draped");
        assert_ne!(frame(&overlay, None), relief, "{what}: own heights");
    }
}

/// A new base on the same grid and color range (another variable plotted)
/// redraws the overlay's cached frame on the new relief, though no
/// uniform changed and the new base's upload count starts over.
#[test]
fn gpu_a_new_base_redraws_the_draped_overlay() {
    let Some((device, queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED gpu_a_new_base_redraws_the_draped_overlay: no GPU device");
        return;
    };
    let surface = |v| {
        Arc::new(Mesh3DRenderer::new_surface(
            &device,
            FORMAT,
            &[v; 16 * 8],
            16,
            8,
        ))
    };
    let (low, high, overlay) = (surface(10.0), surface(100.0), surface(0.0));
    let frame = |r, heights| mask(&pixels(&device, &queue, &draw(r, (false, 2), heights)));
    let on_low = frame(&overlay, Some(&low));
    assert_eq!(on_low, frame(&low, None), "draped on the first base");
    assert_eq!(
        frame(&overlay, Some(&high)),
        frame(&high, None),
        "on the new one"
    );
}
