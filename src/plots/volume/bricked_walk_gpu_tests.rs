//! GPU test of the bricked voxel walk while moving (`walk_bricked.wgsl`):
//! level-0 bricks drawn with coarser dims walk the coarser grid, as the
//! dense volume of that resolution. Skipped (with a note) without an adapter.

use super::VolumeUniformParams;
use super::bricked_gpu_tests::{N, bricked, coarse_dense, mean_diff};
use super::gpu_render::{gpu, params, renderer_wh};
use super::gpu_tests::render;
use crate::utils::colormap::registry;

#[test]
fn gpu_walks_while_moving_take_the_coarser_grid() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED gpu_walks_while_moving_take_the_coarser_grid: no GPU adapter");
        return;
    };
    let dense = renderer_wh(&device, &queue, &coarse_dense(), [N / 2, N / 2], filterable);
    for algorithm in [1, 4] {
        let p = VolumeUniformParams {
            lighting: false,
            ..params(algorithm, 1.0, 0)
        };
        let expected = render(&device, &queue, &dense, &p);
        // Level-0 bricks drawn with half-resolution dims, as while moving.
        let moving = bricked(&device, &queue, false);
        moving
            .bricks()
            .expect("bricked")
            .set_dims([N as u32 / 2; 3]);
        let close = mean_diff(&expected, &render(&device, &queue, &moving, &p));
        eprintln!("mode {algorithm}: level 0 walked on a 2x grid vs dense half: {close}");
        assert!(
            close < 1.5,
            "mode {algorithm}: coarse walk differs by {close}"
        );
    }
}
