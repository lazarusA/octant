//! GPU tests of the slicer's cut through the volume: its box limits the
//! rays, its planes draw opaque orthoslices. Skipped (with a note) without
//! an adapter.

use super::gpu_render::{gpu, params};
use super::gpu_tests::{blob, mean_alpha, render, renderer};
use crate::plots::slice::SliceUniform;
use crate::utils::colormap::registry;

fn boxed(lo: [f32; 3], hi: [f32; 3]) -> SliceUniform {
    SliceUniform {
        lo: [lo[0], lo[1], lo[2], SliceUniform::BOX],
        hi: [hi[0], hi[1], hi[2], 0.0],
    }
}

#[test]
fn gpu_slicer_box_and_planes() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED gpu_slicer_box_and_planes: no GPU adapter, nothing was compared");
        return;
    };
    let volume = renderer(&device, &queue, &blob(), filterable);
    let draw = |slice| {
        let p = super::VolumeUniformParams {
            slice,
            ..params(0, 1.0, 0)
        };
        render(&device, &queue, &volume, &p)
    };
    let whole = mean_alpha(&draw(SliceUniform::OFF));
    let half = mean_alpha(&draw(boxed([0.0; 3], [0.5, 1.0, 1.0])));
    let sliver = mean_alpha(&draw(boxed([0.49, 0.0, 0.0], [0.51, 1.0, 1.0])));
    assert!(whole > 0.01, "the blob must show: {whole}");
    assert!(
        half < 0.8 * whole && half > 0.1 * whole,
        "half cut: {half} of {whole}"
    );
    assert!(sliver < half, "a sliver keeps less: {sliver} vs {half}");

    let planes = |at: [f32; 3]| SliceUniform {
        lo: [at[0], at[1], at[2], SliceUniform::PLANES],
        hi: [1.0, 1.0, 1.0, 0.0],
    };
    let none = draw(planes([-1.0; 3]));
    assert_eq!(mean_alpha(&none), 0.0, "no plane shown draws nothing");
    let one = draw(planes([0.5, -1.0, -1.0]));
    let opaque = one
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] == 255)
        .count();
    assert!(
        opaque > 50,
        "a plane through the blob draws opaque pixels: {opaque}"
    );
    let two = draw(planes([0.5, 0.5, -1.0]));
    assert!(
        mean_alpha(&two) > mean_alpha(&one),
        "a second plane adds pixels"
    );
}
