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
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

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

    // Map quad vertices [-1..1] to tile sub-region [tile_bounds.x..tile_bounds.z] in data space
    let tile_u = mix(uniforms.tile_bounds.x, uniforms.tile_bounds.z, (model.position.x + 1.0) * 0.5);
    let tile_v = mix(uniforms.tile_bounds.y, uniforms.tile_bounds.w, (1.0 - model.position.y) * 0.5);

    let model_x = tile_u * 2.0 - 1.0;
    let model_y = 1.0 - tile_v * 2.0;
    let model_pos = vec2<f32>(model_x, model_y);

    let scaled_model_pos = model_pos * uniforms.aspect_scale;
    let transformed_pos = scaled_model_pos * uniforms.zoom + uniforms.pan;
    out.position = vec4<f32>(transformed_pos, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.uv.x < 0.0 || in.uv.x > 1.0 || in.uv.y < 0.0 || in.uv.y > 1.0) {
        discard;
    }
    let w = max(uniforms.width, 1u);
    let h = max(uniforms.height, 1u);
    let total_elements = w * h;
    let max_idx = min(total_elements, arrayLength(&data_buffer)) - 1u;

    var cell_index: u32 = 0u;
    if (uniforms.coord_mode == 4u || uniforms.coord_mode == 5u) {
        let lon = (in.uv.x - 0.5) * 6.2831853;
        let lat = (0.5 - in.uv.y) * 3.14159265;
        let npix = max(total_elements, 12u);
        let nside = max(u32(round(sqrt(f32(npix) / 12.0))), 1u);
        var pix = healpix_ang2pix_ring(nside, lon, lat);
        if (uniforms.coord_mode == 5u) {
            pix = healpix_ring2nest(nside, pix);
        }
        cell_index = min(pix, max_idx);
    } else if (uniforms.coord_mode == 2u) {
        let max_lx = max(uniforms.lut_size_x, 1u) - 1u;
        let lut_x = clamp(u32(in.uv.x * f32(max_lx) + 0.5), 0u, max_lx);
        let gx = min(u32(coord_x_buffer[lut_x]), w - 1u);

        let max_ly = max(uniforms.lut_size_y, 1u) - 1u;
        let lut_y = clamp(u32(in.uv.y * f32(max_ly) + 0.5), 0u, max_ly);
        let gy = min(u32(coord_y_buffer[lut_y]), h - 1u);
        cell_index = min(gy * w + gx, max_idx);
    } else {
        let gx = clamp(u32(in.uv.x * f32(w)), 0u, w - 1u);
        let gy = clamp(u32(in.uv.y * f32(h)), 0u, h - 1u);
        cell_index = min(gy * w + gx, max_idx);
    }

    let val = data_buffer[cell_index];

    let eval_color = evaluate_plot_color(val, uniforms.color);
    if (eval_color.a <= 0.0) {
        discard;
    }
    return eval_color;
}
