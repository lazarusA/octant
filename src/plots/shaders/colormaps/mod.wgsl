// Shared Colormaps & Data Clipping Evaluator

struct ColorUniforms {
    colormap: u32,
    cmin: f32,
    cmax: f32,
    use_nan_color: u32,
    use_lowclip: u32,
    use_highclip: u32,
    scale_type: u32,
    scale_param: f32,
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

fn evaluate_scaled_norm(val: f32, cmin: f32, cmax: f32, scale_type: u32, scale_param: f32) -> f32 {
    let range = max(cmax - cmin, 1e-30);

    // 0: Linear
    if (scale_type == 0u) {
        return clamp((val - cmin) / range, 0.0, 1.0);
    }

    // 1: Strict Logarithmic (strictly positive data, with numerical threshold for float noise <= 1e-15)
    if (scale_type == 1u) {
        let safe_min = select(cmin, min(1e-12, cmax * 1e-6), cmin <= 1e-15);
        let safe_max = max(cmax, safe_min * 1.0001);
        if (val <= safe_min) {
            return 0.0;
        }
        let safe_v = clamp(val, safe_min, safe_max);

        let log_v = log(safe_v);
        let log_min = log(safe_min);
        let log_max = log(safe_max);
        let log_range = max(log_max - log_min, 1e-6);

        let norm_log = clamp((log_v - log_min) / log_range, 0.0, 1.0);
        let gamma = select(1.0, scale_param, scale_param > 0.0 && scale_param != 1.0);
        return pow(norm_log, gamma);
    }

    // 2: Symlog / Log-Offset
    if (scale_type == 2u) {
        let c = select(1.0, scale_param, scale_param > 0.0);
        let norm_x = clamp((val - cmin) / range, 0.0, 1.0);
        let safe_range = max(abs(range), 1e-6);
        let num = log(c + norm_x * safe_range) - log(c);
        let denom = log(c + safe_range) - log(c);
        return select(norm_x, clamp(num / denom, 0.0, 1.0), denom != 0.0);
    }

    // 3: Sqrt / Diverging
    if (scale_type == 3u) {
        let norm_x = clamp((val - cmin) / range, 0.0, 1.0);
        let x_centered = 2.0 * norm_x - 1.0;
        return clamp(0.5 + 0.5 * sign(x_centered) * sqrt(abs(x_centered)), 0.0, 1.0);
    }

    // 4: Exponential
    if (scale_type == 4u) {
        let norm_x = clamp((val - cmin) / range, 0.0, 1.0);
        let k = select(3.0, scale_param, scale_param > 0.0);
        let num = exp(norm_x * k) - 1.0;
        let denom = exp(k) - 1.0;
        return select(norm_x, clamp(num / denom, 0.0, 1.0), abs(denom) > 1e-5);
    }

    return clamp((val - cmin) / range, 0.0, 1.0);
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

    // 4. In-bounds colormap sampling with direct scaling
    let scaled_val = evaluate_scaled_norm(val, color.cmin, color.cmax, color.scale_type, color.scale_param);
    return vec4<f32>(sample_colormap(color.colormap, scaled_val), 1.0);
}
