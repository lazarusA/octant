// Shared Colormaps & Data Clipping Evaluator

struct ColorUniforms {
    colormap: u32,
    cmin: f32,
    cmax: f32,
    use_nan_color: u32,
    use_lowclip: u32,
    use_highclip: u32,
    _pad0: u32,
    _pad1: u32,
    nan_color: vec4<f32>,
    lowclip_color: vec4<f32>,
    highclip_color: vec4<f32>,
};

fn sample_colormap(colormap_id: u32, t: f32) -> vec3<f32> {
    let norm = clamp(t, 0.0, 1.0);
    if (colormap_id == 0u) {
        return colormap_viridis(norm);
    } else if (colormap_id == 1u) {
        return colormap_plasma(norm);
    } else if (colormap_id == 2u) {
        return colormap_inferno(norm);
    } else if (colormap_id == 3u) {
        return colormap_magma(norm);
    } else if (colormap_id == 4u) {
        return colormap_turbo(norm);
    } else if (colormap_id == 5u) {
        return colormap_coolwarm(norm);
    } else if (colormap_id == 6u) {
        return colormap_cividis(norm);
    }
    return colormap_viridis(norm);
}

fn evaluate_plot_color(val: f32, color: ColorUniforms) -> vec4<f32> {
    // 1. Detect NaN / Inf inputs or corrupt float samples
    if (val != val || abs(val) > 1e30) {
        return select(vec4<f32>(0.0, 0.0, 0.0, 0.0), color.nan_color, color.use_nan_color == 1u);
    }

    // 2. Values below cmin (Lowclip)
    if (val < color.cmin) {
        let default_low = vec4<f32>(sample_colormap(color.colormap, 0.0), 1.0);
        return select(default_low, color.lowclip_color, color.use_lowclip == 1u);
    }

    // 3. Values above cmax (Highclip)
    if (val > color.cmax) {
        let default_high = vec4<f32>(sample_colormap(color.colormap, 1.0), 1.0);
        return select(default_high, color.highclip_color, color.use_highclip == 1u);
    }

    // 4. In-bounds normalized colormap sampling
    let range = max(color.cmax - color.cmin, 1e-6);
    let norm_val = clamp((val - color.cmin) / range, 0.0, 1.0);
    return vec4<f32>(sample_colormap(color.colormap, norm_val), 1.0);
}
