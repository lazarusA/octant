// Volume sampling: unit-box positions to texture coordinates, trilinear and
// exact reads, and the transfer function.

// Returned for missing voxels; every mode treats |v| > 1e30 as missing.
const MISSING: f32 = 3.0e38;

// World length user colors' alpha (NaN and clip colors) is defined for: 64
// samples across a unit box.
const REFERENCE_STEP: f32 = 1.0 / 64.0;

// A ray's samples: `count` steps of `step` (unit-box units) from `start`, each
// `step_world` long in world units. `entry` is where the ray enters the volume
// and `entry_normal` the world normal of the face there (facing the viewer);
// `view` points toward the viewer and `light` toward the headlight (world).
struct Ray {
    start: vec3<f32>,
    step: vec3<f32>,
    count: i32,
    step_world: f32,
    entry: vec3<f32>,
    entry_normal: vec3<f32>,
    view: vec3<f32>,
    light: vec3<f32>,
};

fn is_missing(v: f32) -> bool {
    return abs(v) > 1e30;
}

fn is_composite() -> bool {
    return uniforms.composite != 0u;
}

fn volume_dims() -> vec3<f32> {
    return vec3<f32>(textureDimensions(volume_values));
}

fn volume_shift() -> vec3<u32> {
    return vec3<u32>(uniforms.shift_x, uniforms.shift_y, uniforms.shift_z);
}

// Unit-box position (x right, y up, z toward the viewer) to logical texture
// coordinates: rows run top-down and planes front to back, as stored.
fn logical_coord(p: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(p.x, 1.0 - p.y, 1.0 - p.z);
}

// Sampler coordinate of `p`: clamped to the first and last texel centers
// (ClampToEdge at the box faces), then rotated by the ring-buffer shift, so the
// Repeat sampler blends across the seam inside the volume only.
fn texture_coord(p: vec3<f32>) -> vec3<f32> {
    let dims = volume_dims();
    let half_texel = 0.5 / dims;
    return clamp(logical_coord(p), half_texel, 1.0 - half_texel) + vec3<f32>(volume_shift()) / dims;
}

fn is_valid_at(uvw: vec3<f32>) -> bool {
    if (uniforms.has_invalid == 0u) {
        return true;
    }
    return textureSampleLevel(volume_validity, volume_sampler, uvw, 0.0).r >= 0.5;
}

// Trilinear scalar at `p`, or MISSING.
fn sample_scalar(p: vec3<f32>) -> f32 {
    let uvw = texture_coord(p);
    if (!is_valid_at(uvw)) {
        return MISSING;
    }
    return fetch_trilinear(uvw).r;
}

// Trilinear RGB composite color (alpha = brightest channel), or zero where missing.
fn sample_composite(p: vec3<f32>) -> vec4<f32> {
    let uvw = texture_coord(p);
    if (!is_valid_at(uvw)) {
        return vec4<f32>(0.0);
    }
    return fetch_trilinear(uvw);
}

fn wrap_texel(i: vec3<i32>, n: vec3<i32>) -> vec3<i32> {
    return ((i % n) + n) % n;
}

// 1 where texel `t` is label foreground (value >= 0.5, or a lit composite), 0
// for background and missing voxels.
fn foreground_texel(t: vec3<i32>) -> f32 {
    if (uniforms.has_invalid != 0u && textureLoad(volume_validity, t, 0).r < 0.5) {
        return 0.0;
    }
    let v = textureLoad(volume_values, t, 0);
    let inside = select(v.r >= 0.5, v.a >= 0.01, is_composite());
    return select(0.0, 1.0, inside);
}

// Label foreground blended trilinearly from the eight surrounding voxels: its
// 0.5 level is a smooth surface through a categorical mask, which plain
// filtering of label values cannot give.
fn mask_filtered(p: vec3<f32>) -> f32 {
    let n = vec3<i32>(textureDimensions(volume_values));
    let c = texture_coord(p) * vec3<f32>(n) - 0.5;
    let base = floor(c);
    let f = c - base;
    let a = wrap_texel(vec3<i32>(base), n);
    let b = wrap_texel(vec3<i32>(base) + vec3<i32>(1), n);
    let y0 = mix(
        mix(foreground_texel(vec3<i32>(a.x, a.y, a.z)), foreground_texel(vec3<i32>(b.x, a.y, a.z)), f.x),
        mix(foreground_texel(vec3<i32>(a.x, b.y, a.z)), foreground_texel(vec3<i32>(b.x, b.y, a.z)), f.x),
        f.y,
    );
    let y1 = mix(
        mix(foreground_texel(vec3<i32>(a.x, a.y, b.z)), foreground_texel(vec3<i32>(b.x, a.y, b.z)), f.x),
        mix(foreground_texel(vec3<i32>(a.x, b.y, b.z)), foreground_texel(vec3<i32>(b.x, b.y, b.z)), f.x),
        f.y,
    );
    return mix(y0, y1, f.z);
}

// Exact value of the voxel containing `p`, or MISSING: labels and palette
// indices must never blend.
fn sample_exact(p: vec3<f32>) -> f32 {
    let dims = vec3<u32>(textureDimensions(volume_values));
    let l = clamp(logical_coord(p), vec3<f32>(0.0), vec3<f32>(1.0));
    let texel = (min(vec3<u32>(l * vec3<f32>(dims)), dims - 1u) + volume_shift()) % dims;
    if (uniforms.has_invalid != 0u && textureLoad(volume_validity, texel, 0).r < 0.5) {
        return MISSING;
    }
    return textureLoad(volume_values, texel, 0).r;
}

// Position of `value` on the color scale, quantized as `evaluate_plot_color`
// does for categorical scales and as `sample_colormap` does for stepped maps.
fn scale_position(value: f32) -> f32 {
    let c = uniforms.color;
    var t = evaluate_scaled_norm(value, c.cmin, c.cmax, c.scale_type, c.scale_param);
    if (c.is_categorical == 1u) {
        let n = f32(max(c.num_categories, 1u));
        t = (floor(clamp(t, 0.0, 0.999999) * n) + 0.5) / n;
    }
    if (c.nearest == 1u) {
        t = floor(clamp(t, 0.0, 1.0) * 255.0 + 0.5) / 255.0;
    }
    return t;
}

// Transfer function at scale position `t`: RGB and DVR opacity, blended between
// texels as `sample_colormap` blends atlas texels.
fn transfer_at(t: f32) -> vec4<f32> {
    let u = (clamp(t, 0.0, 1.0) * 255.0 + 0.5) / 256.0;
    return textureSampleLevel(transfer_lut, volume_sampler, vec2<f32>(u, 0.5), 0.0);
}

// Display color of `value` (opaque in range), with NaN and clip colors as in
// `evaluate_plot_color`.
fn value_color(value: f32) -> vec4<f32> {
    let c = uniforms.color;
    if (is_missing(value)) {
        return select(vec4<f32>(0.0), c.nan_color, c.use_nan_color == 1u);
    }
    if (value < c.cmin) {
        return select(vec4<f32>(transfer_at(0.0).rgb, 1.0), c.lowclip_color, c.use_lowclip == 1u);
    }
    if (value > c.cmax) {
        return select(vec4<f32>(transfer_at(1.0).rgb, 1.0), c.highclip_color, c.use_highclip == 1u);
    }
    return vec4<f32>(transfer_at(scale_position(value)).rgb, 1.0);
}

// Opacity of one step of `step_world` for a sample whose opacity is defined
// per REFERENCE_STEP: view- and quality-independent accumulation.
fn corrected_alpha(alpha: f32, step_world: f32) -> f32 {
    return 1.0 - pow(1.0 - clamp(alpha, 0.0, 0.9999), step_world / REFERENCE_STEP);
}

// Beer-Lambert opacity of one step through extinction `Density * weight` per
// world unit: Density is the optical depth across one world unit (about the
// volume's longest side) at weight 1.
fn extinction_alpha(weight: f32, step_world: f32) -> f32 {
    return 1.0 - exp(-uniforms.absorption * max(weight, 0.0) * step_world);
}

fn premultiply(c: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(c.rgb * c.a, c.a);
}
