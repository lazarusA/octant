//! GPU tests of resting frames built in bands of rows
//! (`render.rs` `render_frame_banded`) over the bricked test volume of
//! `bricked_gpu_tests.rs`. Skipped (with a note) without an adapter.

use super::VolumeUniformParams;
use super::bricked_gpu_tests::bricked;
use super::gpu_render::{gpu, params, render_sized};
use crate::utils::colormap::registry;

/// Shows a whole frame of `p` first: with nothing shown, the renderer draws
/// whole instead of in bands.
fn prime(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &super::VolumeRenderer,
    p: &VolumeUniformParams,
    resources: &eframe::egui_wgpu::CallbackResources,
) {
    let mut encoder = device.create_command_encoder(&Default::default());
    renderer.render_frame_banded(device, queue, &mut encoder, (p, [64, 64], 1), resources);
    queue.submit([encoder.finish()]);
}

#[test]
fn gpu_the_first_frame_is_drawn_whole() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_the_first_frame_is_drawn_whole: no GPU adapter");
        return;
    };
    let p = params(0, 1.0, 0);
    let renderer = bricked(&device, &queue, false);
    let resources = super::gpu_render::atlas(&device, &queue);
    let mut encoder = device.create_command_encoder(&Default::default());
    renderer.render_frame_banded(&device, &queue, &mut encoder, (&p, [64, 64], 4), &resources);
    queue.submit([encoder.finish()]);
    assert!(renderer.frame_texture().is_some(), "never an empty canvas");
    assert!(renderer.shows(&p, 1));
    renderer.forget_frames();
    assert!(
        renderer.frame_texture().is_none(),
        "a reused renderer starts clean"
    );
}

#[test]
fn gpu_a_frame_built_in_bands_equals_the_whole() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_a_frame_built_in_bands_equals_the_whole: no GPU adapter");
        return;
    };
    let size = 64;
    let p = params(0, 1.0, 0);
    let whole = render_sized(&device, &queue, &bricked(&device, &queue, false), &p, size);
    let banded = bricked(&device, &queue, false);
    let resources = super::gpu_render::atlas(&device, &queue);
    prime(&device, &queue, &banded, &p, &resources);
    for band in 0..4 {
        assert!(
            !banded.shows(&p, 4),
            "band {band}: not shown before the last"
        );
        let mut encoder = device.create_command_encoder(&Default::default());
        let target = (&p, [size, size], 4);
        assert!(banded.render_frame_banded(&device, &queue, &mut encoder, target, &resources));
        queue.submit([encoder.finish()]);
    }
    assert!(banded.shows(&p, 4));
    // Already shown: read back without rendering again.
    let built = render_sized(&device, &queue, &banded, &p, size);
    assert_eq!(built, whole);
}

#[test]
fn gpu_bricks_arriving_mid_build_still_let_the_frame_finish() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!(
            "SKIPPED gpu_bricks_arriving_mid_build_still_let_the_frame_finish: no GPU adapter"
        );
        return;
    };
    let p = params(0, 1.0, 0);
    let renderer = bricked(&device, &queue, false);
    let bricks = renderer.bricks().expect("bricked");
    let resources = super::gpu_render::atlas(&device, &queue);
    let band = |renderer: &super::VolumeRenderer| {
        let mut encoder = device.create_command_encoder(&Default::default());
        renderer.render_frame_banded(&device, &queue, &mut encoder, (&p, [64, 64], 4), &resources);
        queue.submit([encoder.finish()]);
    };
    prime(&device, &queue, &renderer, &p, &resources);
    for b in 0..4 {
        // A batch of bricks lands before every band (the data version moves).
        bricks.write_range(&queue, [0, 0, 0], [0.0, 1.0], false);
        band(&renderer);
        assert_eq!(renderer.building(), b < 3, "finished at band {b}");
    }
    // Its first bands drew older bricks: stale, built once more, then shown.
    assert!(
        !renderer.shows(&p, 4),
        "a frame bricks joined mid-build is stale"
    );
    for b in 0..4 {
        band(&renderer);
        assert_eq!(renderer.shows(&p, 4), b == 3, "rebuilt band {b}");
    }
}

#[test]
fn gpu_rays_report_the_table_cells_they_reach() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_rays_report_the_table_cells_they_reach: no GPU adapter");
        return;
    };
    let resources = super::gpu_render::atlas(&device, &queue);
    // Opaque, rays stop where they enter the box: some cells stay unseen;
    // see-through, they cross every cell.
    let mut counts = Vec::new();
    for transparency in [false, true] {
        let p = VolumeUniformParams {
            transparency,
            opacity_scale: 0.05,
            ..params(0, 1.0, 0)
        };
        let renderer = bricked(&device, &queue, false);
        let feedback = &renderer.bricks().expect("bricked").feedback;
        prime(&device, &queue, &renderer, &p, &resources);
        for _ in 0..2 {
            let mut encoder = device.create_command_encoder(&Default::default());
            renderer.render_frame_banded(
                &device,
                &queue,
                &mut encoder,
                (&p, [64, 64], 2),
                &resources,
            );
            queue.submit([encoder.finish()]);
        }
        let mut taken = None;
        for _ in 0..100 {
            feedback.poll(&device, &queue);
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            taken = feedback.take();
            if taken.is_some() {
                break;
            }
        }
        let (frame, seen) = taken.expect("read back");
        assert!(frame == p, "tagged with its frame");
        counts.push(seen.count());
    }
    assert_eq!(counts[1], 8, "see-through reaches all: {counts:?}");
    assert!(counts[0] < counts[1], "opaque hides some: {counts:?}");
}
