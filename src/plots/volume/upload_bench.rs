//! Timing of the per-step work of animating a large volume (1440×720×32, the
//! depth the app keeps for a 0.25° global grid): the texture update every new
//! timestep pays, and a frame. Ignored; run with
//! `cargo test --release --lib volume_upload_benchmark -- --ignored --nocapture`.

use super::gpu_render::{gpu, params, renderer_wh};
use std::time::Instant;

#[test]
#[ignore = "timing benchmark"]
fn volume_upload_benchmark() {
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED volume_upload_benchmark: no GPU adapter");
        return;
    };
    let (w, h, d) = (1440usize, 720usize, 32usize);
    let values: Vec<f32> = (0..w * h * d)
        .map(|i| ((i % 977) as f32 * 0.01).sin())
        .collect();
    let renderer = renderer_wh(&device, &queue, &values, [w, h], filterable);
    let steps = 4;
    let start = Instant::now();
    for _ in 0..steps {
        renderer.update_data(&queue, &values);
    }
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let per_step = start.elapsed().as_secs_f64() * 1e3 / f64::from(steps);
    eprintln!("texture update per timestep: {per_step:.0} ms");

    let mut p = params(0, 1.0, 0);
    (p.aspect_y, p.aspect_z, p.rot_x) = (0.5, 0.2, 0.6);
    let frames = 6;
    let start = Instant::now();
    for f in 0..frames {
        p.rot_y = 0.3 + f as f32 * 0.01;
        let mut encoder = device.create_command_encoder(&Default::default());
        renderer.render_frame(&device, &queue, &mut encoder, &p, [1600, 1000]);
        queue.submit([encoder.finish()]);
    }
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let per_frame = start.elapsed().as_secs_f64() * 1e3 / f64::from(frames);
    eprintln!("DVR frame (1600x1000): {per_frame:.1} ms");
}
