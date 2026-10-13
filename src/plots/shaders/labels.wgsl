// Label images (segmentations): each label id drawn in its own color.
//
// Group 0 is the heatmap's (uniforms, data, coordinate lookup tables), with
// the data read as `u32` label ids (stored as the bits of `f32`s). Only the
// leading heatmap uniforms are declared; the color parameters after them are
// not used. Group 1 holds the label colors.

struct Uniforms {
    pan: vec2<f32>,
    zoom: f32,
    coord_mode: u32,
    aspect_scale: vec2<f32>,
    width: u32,
    height: u32,
    tile_bounds: vec4<f32>,
    lut_size_x: u32,
    lut_size_y: u32,
    _pad0: u32,
    _pad1: u32,
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

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let tile_u = mix(uniforms.tile_bounds.x, uniforms.tile_bounds.z, (model.position.x + 1.0) * 0.5);
    let tile_v = mix(uniforms.tile_bounds.y, uniforms.tile_bounds.w, (1.0 - model.position.y) * 0.5);
    let model_pos = vec2<f32>(tile_u * 2.0 - 1.0, 1.0 - tile_v * 2.0);
    let transformed = model_pos * uniforms.aspect_scale * uniforms.zoom + uniforms.pan;
    out.position = vec4<f32>(transformed, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}

// The label id under `uv`; 0 (background) outside the image.
fn label_at(uv: vec2<f32>) -> u32 {
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return 0u;
    }
    let w = max(uniforms.width, 1u);
    let h = max(uniforms.height, 1u);
    let max_idx = min(w * h, arrayLength(&label_buffer)) - 1u;
    var gx: u32;
    var gy: u32;
    if (uniforms.coord_mode == 2u) {
        let max_lx = max(uniforms.lut_size_x, 1u) - 1u;
        let max_ly = max(uniforms.lut_size_y, 1u) - 1u;
        gx = min(u32(coord_x_buffer[clamp(u32(uv.x * f32(max_lx) + 0.5), 0u, max_lx)]), w - 1u);
        gy = min(u32(coord_y_buffer[clamp(u32(uv.y * f32(max_ly) + 0.5), 0u, max_ly)]), h - 1u);
    } else {
        gx = clamp(u32(uv.x * f32(w)), 0u, w - 1u);
        gy = clamp(u32(uv.y * f32(h)), 0u, h - 1u);
    }
    return label_buffer[min(gy * w + gx, max_idx)];
}

// PCG hash, mirrored by `label_hash` in `labels.rs`.
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
    if (in.uv.x < 0.0 || in.uv.x > 1.0 || in.uv.y < 0.0 || in.uv.y > 1.0) {
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
