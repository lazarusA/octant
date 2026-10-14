// Label images (segmentations): each label id drawn in its own color.
//
// Group 0 is the heatmap's (uniforms, data, coordinate lookup tables), with
// the data read as `u32` label ids (stored as the bits of `f32`s). Only the
// leading quad uniforms (`common/quad.wgsl`) are declared; the color
// parameters after them are not used. Group 1 holds the label colors.

struct Uniforms {
    quad: QuadUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> label_buffer: array<u32>;

struct LabelParams {
    // Entries of `palette`.
    count: u32,
    // 1 draws only label boundaries.
    outline: u32,
    opacity: f32,
    _pad: u32,
};

@group(1) @binding(0)
var<uniform> params: LabelParams;

// `(label id, RGBA8 packed red-first)`, sorted by id.
@group(1) @binding(1)
var<storage, read> palette: array<vec2<u32>>;

// The label id under `uv`; 0 (background) outside the image.
fn label_at(uv: vec2<f32>) -> u32 {
    if (outside_quad(uv)) {
        return 0u;
    }
    return label_buffer[quad_cell(uv, arrayLength(&label_buffer))];
}

// PCG hash, mirrored by `label_hash` in `label_palette.rs`.
fn label_hash(v: u32) -> u32 {
    let state = v * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let k = vec3<f32>(5.0, 3.0, 1.0);
    let p = clamp(abs(fract(vec3<f32>(h) + k / 6.0) * 6.0 - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
    return v * mix(vec3<f32>(1.0), p, s);
}

// The color recorded for `id`, else a stable color from its hash.
fn label_color(id: u32) -> vec4<f32> {
    var lo = 0u;
    var hi = params.count;
    while (lo < hi) {
        let mid = (lo + hi) / 2u;
        let entry = palette[mid];
        if (entry.x == id) {
            return unpack4x8unorm(entry.y);
        }
        if (entry.x < id) {
            lo = mid + 1u;
        } else {
            hi = mid;
        }
    }
    let hue = f32(label_hash(id) & 0xFFFFu) / 65535.0;
    return vec4<f32>(hsv_to_rgb(hue, 0.65, 0.95), 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // One screen pixel in uv, taken before any branch (uniform control flow).
    let du = dpdx(in.uv);
    let dv = dpdy(in.uv);
    if (outside_quad(in.uv)) {
        discard;
    }
    let id = label_at(in.uv);
    if (id == 0u) {
        discard;
    }
    if (params.outline == 1u) {
        let edge = label_at(in.uv + du) != id || label_at(in.uv - du) != id
            || label_at(in.uv + dv) != id || label_at(in.uv - dv) != id;
        if (!edge) {
            discard;
        }
    }
    let color = label_color(id);
    let alpha = color.a * params.opacity;
    if (alpha <= 0.0) {
        discard;
    }
    return vec4<f32>(color.rgb, alpha);
}
