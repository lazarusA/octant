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
    is_categorical: u32,
    num_categories: u32,
    reverse: u32,
    nearest: u32,
    fallback_colormap: u32,
    opacity: f32,
    _pad1: u32,
    _pad2: u32,
    nan_color: vec4<f32>,
    lowclip_color: vec4<f32>,
    highclip_color: vec4<f32>,
};

// Uniform `colormap` value selecting direct RGB composite (truecolor) rendering.
const COLORMAP_RGB_COMPOSITE: u32 = 0xFFFFFFFFu;

// Shared colormap atlas: one 256-texel RGBA8 row per colormap, written from the
// CPU registry (`src/utils/colormap`), which samples the same rows identically.
@group(1) @binding(0)
var colormap_lut: texture_2d<f32>;

// Samples row `colormap_id` at t in [0, 1] by blending the two neighbouring
// texels, or with the nearest texel for stepped (categorical) maps.
fn sample_colormap(colormap_id: u32, t: f32, nearest: bool) -> vec3<f32> {
    let rows = textureDimensions(colormap_lut).y;
    let row = i32(min(colormap_id, rows - 1u));
    // NaN samples the start, as on the CPU; infinities clamp to the ends. NaN is
    // detected from its bits: WGSL may fold float comparisons with NaN.
    let is_nan = (bitcast<u32>(t) & 0x7fffffffu) > 0x7f800000u;
    let x = clamp(select(t, 0.0, is_nan), 0.0, 1.0) * 255.0;
    if (nearest) {
        let i = min(u32(floor(x + 0.5)), 255u);
        return textureLoad(colormap_lut, vec2<i32>(i32(i), row), 0).rgb;
    }
    let i0 = min(u32(floor(x)), 254u);
    let f = x - f32(i0);
    let c0 = textureLoad(colormap_lut, vec2<i32>(i32(i0), row), 0).rgb;
    let c1 = textureLoad(colormap_lut, vec2<i32>(i32(i0) + 1, row), 0).rgb;
    return mix(c0, c1, f);
}

// Row of the plot's colormap; RGB composite mode has none, so colormap-only
// paths (per-line colors, indexed volumes) draw the fallback row.
fn plot_colormap_row(color: ColorUniforms) -> u32 {
    return select(color.colormap, color.fallback_colormap, color.colormap == COLORMAP_RGB_COMPOSITE);
}

// Samples the plot's active colormap, honoring the reversed flag.
fn sample_plot_colormap(color: ColorUniforms, t: f32) -> vec3<f32> {
    return sample_colormap(plot_colormap_row(color), select(t, 1.0 - t, color.reverse == 1u), color.nearest == 1u);
}

fn evaluate_scaled_norm(val: f32, cmin: f32, cmax: f32, scale_type: u32, scale_param: f32) -> f32 {
    let range = max(cmax - cmin, 1e-30);

    // 0: Linear
    if (scale_type == 0u) {
        return clamp((val - cmin) / range, 0.0, 1.0);
    }

    // 1: Strict Logarithmic (strictly positive data, with numerical threshold for float noise <= 1e-15)
    if (scale_type == 1u) {
        if (cmin < -1e-15) {
            return clamp((val - cmin) / range, 0.0, 1.0);
        }
        let safe_min = select(cmin, max(cmax * 0.001, 1e-12), cmin <= 1e-15);
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
    // 1. Detect NaN / Inf inputs using IEEE-754 exponent bits (0x7F800000) or val != val
    let bits = bitcast<u32>(val);
    let is_ieee_nan_inf = (bits & 0x7F800000u) == 0x7F800000u;
    if (is_ieee_nan_inf || val != val || abs(val) > 1e30) {
        return select(vec4<f32>(0.0, 0.0, 0.0, 0.0), color.nan_color, color.use_nan_color == 1u);
    }

    // Direct RGB Composite truecolor mode
    if (color.colormap == COLORMAP_RGB_COMPOSITE) {
        let packed = u32(val);
        let r = f32(packed & 0xFFu) / 255.0;
        let g = f32((packed >> 8u) & 0xFFu) / 255.0;
        let b = f32((packed >> 16u) & 0xFFu) / 255.0;
        return vec4<f32>(r, g, b, 1.0);
    }

    // 2. Values below cmin (Lowclip)
    if (val < color.cmin) {
        let default_low = vec4<f32>(sample_plot_colormap(color, 0.0), color.opacity);
        return select(default_low, color.lowclip_color, color.use_lowclip == 1u);
    }

    // 3. Values above cmax (Highclip)
    if (val > color.cmax) {
        let default_high = vec4<f32>(sample_plot_colormap(color, 1.0), color.opacity);
        return select(default_high, color.highclip_color, color.use_highclip == 1u);
    }

    // 4. In-bounds colormap sampling with direct scaling and optional categorical quantization
    var scaled_val = evaluate_scaled_norm(val, color.cmin, color.cmax, color.scale_type, color.scale_param);

    if (color.is_categorical == 1u) {
        let num_cats = f32(max(color.num_categories, 1u));
        let bin_idx = floor(clamp(scaled_val, 0.0, 0.999999) * num_cats);
        scaled_val = (bin_idx + 0.5) / num_cats;
    }

    return vec4<f32>(sample_plot_colormap(color, scaled_val), color.opacity);
}
