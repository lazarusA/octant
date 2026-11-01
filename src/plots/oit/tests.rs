//! GPU checks of the OIT passes and composite: draw order does not change the
//! result, translucent layers cover by 1 - Π(1 - α), and opaque layers hide
//! what is behind them. Skipped (passes, with a message) without a GPU.

use super::test_scene::{SIZE, gpu, key_of, near, render, scene};

#[test]
fn oit_is_order_independent_and_hides_behind_opaque() {
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIPPED oit_is_order_independent_and_hides_behind_opaque: no OIT-capable GPU");
        return;
    };
    let mut scene = scene(&device, &queue);
    let reference = render(&device, &queue, &mut scene, &[0, 1, 2, 3]);
    for order in [[3, 2, 1, 0], [1, 3, 0, 2], [2, 0, 3, 1]] {
        let pixel = render(&device, &queue, &mut scene, &order);
        for (c, (p, r)) in pixel.iter().zip(reference).enumerate() {
            assert!(
                p.abs_diff(r) <= 1,
                "order {order:?} channel {c}: {p} vs {r}"
            );
        }
    }
    // Red and blue (equal weights this close) cover 1 - 0.5 * 0.5 = 0.75 over
    // opaque green; the white layer behind green adds nothing.
    let [r, g, b, a] = reference;
    assert!(near(r, 0.375 * 255.0), "red {r}");
    assert!(near(b, 0.375 * 255.0), "blue {b}");
    assert!(near(g, 0.25 * 255.0), "green {g}");
    assert!(near(a, 255.0), "alpha {a}");
}

#[test]
fn oit_without_opaque_layers_keeps_coverage() {
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIPPED oit_without_opaque_layers_keeps_coverage: no OIT-capable GPU");
        return;
    };
    let mut scene = scene(&device, &queue);
    // Red alone: premultiplied (0.5, 0, 0, 0.5).
    let first = render(&device, &queue, &mut scene, &[0]);
    let [r, g, b, a] = first;
    assert!(near(r, 127.5) && near(g, 0.0) && near(b, 0.0) && near(a, 127.5));
    // A released frame is allocated again on the next render.
    scene.state.release_frame();
    assert_eq!(render(&device, &queue, &mut scene, &[0]), first);
}

#[test]
fn oit_many_near_layers_do_not_overflow() {
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIPPED oit_many_near_layers_do_not_overflow: no OIT-capable GPU");
        return;
    };
    let mut scene = scene(&device, &queue);
    // 200 mid-gray layers at the weight cap: an overflowed half-float sum
    // would saturate color and alpha alike and turn the gray white.
    let [r, g, b, a] = render(&device, &queue, &mut scene, &[4; 200]);
    for (name, c) in [("red", r), ("green", g), ("blue", b)] {
        assert!(near(c, 127.5), "{name} {c}");
    }
    assert!(near(a, 255.0), "alpha {a}");
}

#[test]
fn an_unchanged_frame_is_not_rendered_again() {
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIPPED an_unchanged_frame_is_not_rendered_again: no GPU device");
        return;
    };
    let mut scene = scene(&device, &queue);
    render(&device, &queue, &mut scene, &[0, 1]);
    let Some(atlas) = crate::plots::colormap_atlas::bind_group(&scene.resources) else {
        panic!("atlas missing");
    };
    let mut encoder = device.create_command_encoder(&Default::default());
    let size_key = ([SIZE; 2], key_of(&[0, 1]));
    scene
        .state
        .render(&device, &mut encoder, size_key, atlas, |_| {
            panic!("same key and size: nothing to draw");
        });
    let drawn = std::cell::Cell::new(false);
    let size_key = ([SIZE; 2], key_of(&[1, 0]));
    scene
        .state
        .render(&device, &mut encoder, size_key, atlas, |_| drawn.set(true));
    assert!(drawn.get(), "a new key draws");
}
