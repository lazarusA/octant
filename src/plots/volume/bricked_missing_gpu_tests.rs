//! Missing voxels in bricks (`sample_bricked.wgsl` `filtered_value`): a
//! brick flagged as missing some weighs its trilinear taps by validity, as
//! the dense volume's validity texture does, instead of letting filtering
//! spread NaN over every sample touching one.

use super::VolumeUniformParams;
use super::bricked_gpu_tests::{N, bricked_of, dense_of, mean_diff};
use super::gpu_render::{gpu, params, renderer_wh};
use super::gpu_tests::render;
use crate::utils::colormap::registry;

/// A smooth field missing one voxel in seven, scattered.
fn holed(x: usize, y: usize, z: usize) -> f32 {
    if (x + 2 * y + 3 * z).is_multiple_of(7) {
        return f32::NAN;
    }
    let f = |i: usize| i as f32 / N as f32;
    0.3 + 0.4 * f(y) + 0.1 * f(x)
}

#[test]
fn gpu_scattered_missing_voxels_draw_as_the_dense_volume_does() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED gpu_scattered_missing_voxels_draw_as_the_dense_volume_does: no GPU");
        return;
    };
    let p = VolumeUniformParams {
        lighting: false,
        ..params(0, 1.0, 0)
    };
    let dense = renderer_wh(&device, &queue, &dense_of(holed, false), [N, N], filterable);
    let expected = render(&device, &queue, &dense, &p);
    let weighed = render(
        &device,
        &queue,
        &bricked_of(&device, &queue, false, (holed, true)),
        &p,
    );
    let spread = render(
        &device,
        &queue,
        &bricked_of(&device, &queue, false, (holed, false)),
        &p,
    );
    let (close, far) = (
        mean_diff(&expected, &weighed),
        mean_diff(&expected, &spread),
    );
    eprintln!("missing voxels: weighed {close}, spread {far}");
    // How filtering treats NaN is the hardware's: `spread` only shows it.
    assert!(close < 1.0, "weighed taps differ from dense by {close}");
}
