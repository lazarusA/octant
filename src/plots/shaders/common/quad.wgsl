// The data quad shared by `heatmap.wgsl` and `labels.wgsl`: its uniforms,
// vertex shader and cell lookup. Each shader binds group 0 binding 0 as
// `uniforms`, with these as its leading `quad` field (the heatmap's color
// parameters follow them), and includes `coords.wgsl`.

struct QuadUniforms {
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
    // The slicer's box over the whole data (`common/slice.wgsl`).
    slice: Slice,
};

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
    let q = uniforms.quad;
    // Quad vertices [-1..1] map to the tile's sub-region of the data space.
    let tile_u = mix(q.tile_bounds.x, q.tile_bounds.z, (model.position.x + 1.0) * 0.5);
    let tile_v = mix(q.tile_bounds.y, q.tile_bounds.w, (1.0 - model.position.y) * 0.5);
    let model_pos = vec2<f32>(tile_u * 2.0 - 1.0, 1.0 - tile_v * 2.0);
    var out: VertexOutput;
    out.position = vec4<f32>(model_pos * q.aspect_scale * q.zoom + q.pan, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}

// Whether `uv` (over this quad, a tile's or the whole data's) falls outside
// the data drawn: off the quad, or cut away by the slicer's box, read over
// the whole data (`tile_bounds` places the quad in it).
fn outside_quad(uv: vec2<f32>) -> bool {
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return true;
    }
    let tb = uniforms.quad.tile_bounds;
    let data = mix(tb.xy, tb.zw, uv);
    return !slice_box_keeps(uniforms.quad.slice, vec3<f32>(data, 0.5));
}

// Index of the cell under `uv` in a buffer of `len` values: through the
// coordinate lookup tables in coord mode 2, else on an even grid.
fn quad_cell(uv: vec2<f32>, len: u32) -> u32 {
    let q = uniforms.quad;
    let w = max(q.width, 1u);
    let h = max(q.height, 1u);
    var gx: u32;
    var gy: u32;
    if (q.coord_mode == 2u) {
        let max_lx = max(q.lut_size_x, 1u) - 1u;
        let max_ly = max(q.lut_size_y, 1u) - 1u;
        gx = min(u32(coord_x_buffer[clamp(u32(uv.x * f32(max_lx) + 0.5), 0u, max_lx)]), w - 1u);
        gy = min(u32(coord_y_buffer[clamp(u32(uv.y * f32(max_ly) + 0.5), 0u, max_ly)]), h - 1u);
    } else {
        gx = clamp(u32(uv.x * f32(w)), 0u, w - 1u);
        gy = clamp(u32(uv.y * f32(h)), 0u, h - 1u);
    }
    return min(gy * w + gx, min(w * h, len) - 1u);
}
