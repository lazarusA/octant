//! GPU/CPU parity: renders WGSL `sample_plot_colormap` from the real atlas into
//! a 256×1 target and compares every pixel with the CPU registry, for
//! continuous, stepped (nearest-texel), smooth-twin, reversed and RGB composite
//! (fallback row) uniforms. The first pixels sample special values (ends, NaN,
//! infinities, out of range); the rest sample pixel centers.
//! Skipped (passes, with a message) when no GPU adapter is available.

use super::ColormapAtlas;
use super::probe::{SPECIAL_T, gpu, render_row};
use crate::plots::common::PlotColorParams;
use crate::utils::colormap::{COLORMAP_RGB_COMPOSITE, LUT_SIZE, orient, registry};

/// Uniforms for row `id` as `get_color_params` builds them.
fn params(id: u32, reversed: bool, composite: bool) -> PlotColorParams {
    PlotColorParams {
        colormap: if composite {
            COLORMAP_RGB_COMPOSITE
        } else {
            id
        },
        fallback_colormap: id,
        nearest: u32::from(registry::is_stepped(id)),
        reverse: u32::from(reversed),
        ..Default::default()
    }
}

fn pixel_t(x: usize) -> f32 {
    SPECIAL_T
        .get(x)
        .copied()
        .unwrap_or((x as f32 + 0.5) / 256.0)
}

#[test]
fn gpu_atlas_matches_cpu_sampling() {
    let _registry = crate::utils::colormap::registry::test_lock();
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIPPED gpu_atlas_matches_cpu_sampling: no GPU adapter, nothing was compared");
        return;
    };
    let atlas = ColormapAtlas::new(&device, &queue);
    let last = u32::try_from(registry::rows() - 1).unwrap_or(0);
    let tab10 = registry::find("classic:tab10").unwrap_or(0);
    let cases = [
        params(registry::default_id(), false, false),
        params(registry::default_id(), true, false),
        params(registry::find("classic:turbo").unwrap_or(0), false, false),
        params(tab10, false, false),
        params(tab10, true, false),
        params(last, false, false),
        params(tab10, true, true),
    ];
    for color in cases {
        let id = color.fallback_colormap;
        let pixels = render_row(&device, &queue, &atlas, &color);
        assert_eq!(pixels.len(), LUT_SIZE * 4, "readback failed");
        for x in 0..LUT_SIZE {
            // Pixel centers fall between LUT texels, exercising blending and nearest.
            let t = orient(pixel_t(x), color.reverse != 0);
            let cpu = registry::sample_row(id, t, color.nearest != 0);
            let gpu = &pixels[x * 4..x * 4 + 3];
            for (c, (g, e)) in gpu.iter().zip([cpu.r(), cpu.g(), cpu.b()]).enumerate() {
                assert!(
                    g.abs_diff(e) <= 1,
                    "map {id} (reverse {}, colormap {:#x}) texel {x} channel {c}: gpu {g} cpu {e}",
                    color.reverse,
                    color.colormap,
                );
            }
        }
    }
}

#[test]
fn gpu_alpha_curve_matches_cpu() {
    use crate::utils::colormap::{AlphaInterp, alpha};
    let _registry = crate::utils::colormap::registry::test_lock();
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIPPED gpu_alpha_curve_matches_cpu: no GPU adapter, nothing was compared");
        return;
    };
    for (text, interp) in [
        ("0.1, 0.9, 0.2, 0.7", AlphaInterp::Linear),
        ("0:1, 0.4:0, 0.6:1", AlphaInterp::Step),
    ] {
        let Ok(Some(curve)) = alpha::parse(text) else {
            panic!("{text} should parse");
        };
        registry::set_alpha_curve(Some(alpha::bake(&curve, interp)));
        let atlas = ColormapAtlas::new(&device, &queue);
        // Reversed: the curve follows the data position, not the colormap.
        let color = PlotColorParams {
            opacity: 0.8,
            alpha_row: registry::alpha_row().unwrap_or(u32::MAX),
            ..params(registry::default_id(), true, false)
        };
        let pixels = render_row(&device, &queue, &atlas, &color);
        registry::set_alpha_curve(None);
        assert_eq!(pixels.len(), LUT_SIZE * 4, "readback failed");
        for x in 0..LUT_SIZE {
            let cpu = (0.8 * alpha_curve_at(&curve, interp, pixel_t(x)) * 255.0).round() as u8;
            let gpu = pixels[x * 4 + 3];
            assert!(
                gpu.abs_diff(cpu) <= 1,
                "{text} texel {x}: gpu {gpu} cpu {cpu}"
            );
        }
    }
}

fn alpha_curve_at(
    curve: &crate::utils::colormap::AlphaCurve,
    interp: crate::utils::colormap::AlphaInterp,
    t: f32,
) -> f32 {
    crate::utils::colormap::lut::sample_alpha(
        &crate::utils::colormap::alpha::bake(curve, interp),
        t,
    )
}
