use egui::Color32;

/// Samples colormap RGB color for normalized parameter t in [0.0, 1.0] matching WGSL shaders 100% bit-exactly.
pub fn sample_colormap_rgb(colormap_id: u32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let (r, g, b) = match colormap_id {
        0 => sample_viridis(t),
        1 => sample_plasma(t),
        2 => sample_inferno(t),
        3 => sample_magma(t),
        4 => sample_turbo(t),
        5 => sample_coolwarm(t),
        6 => sample_cividis(t),
        _ => sample_viridis(t),
    };

    Color32::from_rgb(
        (r * 255.0).clamp(0.0, 255.0) as u8,
        (g * 255.0).clamp(0.0, 255.0) as u8,
        (b * 255.0).clamp(0.0, 255.0) as u8,
    )
}

fn mix_rgb(c0: [f32; 3], c1: [f32; 3], t: f32) -> (f32, f32, f32) {
    (
        c0[0] + (c1[0] - c0[0]) * t,
        c0[1] + (c1[1] - c0[1]) * t,
        c0[2] + (c1[2] - c0[2]) * t,
    )
}

fn sample_segmented(x: f32, c0: [f32; 3], c1: [f32; 3], c2: [f32; 3], c3: [f32; 3], c4: [f32; 3]) -> (f32, f32, f32) {
    if x < 0.25 {
        mix_rgb(c0, c1, x / 0.25)
    } else if x < 0.50 {
        mix_rgb(c1, c2, (x - 0.25) / 0.25)
    } else if x < 0.75 {
        mix_rgb(c2, c3, (x - 0.50) / 0.25)
    } else {
        mix_rgb(c3, c4, (x - 0.75) / 0.25)
    }
}

// 0: Viridis (viridis.wgsl)
fn sample_viridis(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.267, 0.004, 0.329],
        [0.231, 0.322, 0.545],
        [0.129, 0.569, 0.551],
        [0.369, 0.788, 0.384],
        [0.992, 0.906, 0.145],
    )
}

// 1: Plasma (plasma.wgsl)
fn sample_plasma(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.051, 0.031, 0.529],
        [0.416, 0.000, 0.659],
        [0.694, 0.165, 0.565],
        [0.882, 0.392, 0.384],
        [0.941, 0.976, 0.129],
    )
}

// 2: Inferno (inferno.wgsl)
fn sample_inferno(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.000, 0.000, 0.016],
        [0.341, 0.062, 0.429],
        [0.733, 0.216, 0.330],
        [0.976, 0.557, 0.035],
        [0.988, 1.000, 0.643],
    )
}

// 3: Magma (magma.wgsl)
fn sample_magma(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.000, 0.000, 0.016],
        [0.318, 0.071, 0.486],
        [0.714, 0.212, 0.475],
        [0.984, 0.533, 0.380],
        [0.988, 0.992, 0.749],
    )
}

// 4: Turbo (turbo.wgsl)
fn sample_turbo(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.190, 0.072, 0.232],
        [0.156, 0.447, 0.996],
        [0.134, 0.887, 0.525],
        [0.925, 0.875, 0.134],
        [0.900, 0.180, 0.090],
    )
}

// 5: Coolwarm (coolwarm.wgsl)
fn sample_coolwarm(t: f32) -> (f32, f32, f32) {
    let cool = [0.230, 0.299, 0.754];
    let mid = [0.865, 0.865, 0.865];
    let warm = [0.706, 0.016, 0.150];

    if t < 0.50 {
        mix_rgb(cool, mid, t / 0.50)
    } else {
        mix_rgb(mid, warm, (t - 0.50) / 0.50)
    }
}

// 6: Cividis (cividis.wgsl)
fn sample_cividis(t: f32) -> (f32, f32, f32) {
    sample_segmented(
        t,
        [0.000, 0.135, 0.304],
        [0.286, 0.337, 0.435],
        [0.506, 0.514, 0.463],
        [0.741, 0.702, 0.430],
        [0.996, 0.906, 0.145],
    )
}
