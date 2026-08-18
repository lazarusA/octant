struct Uniforms {
    pan: vec2<f32>,
    zoom: f32,
    _pad: u32,
    aspect_scale: vec2<f32>,
    width: u32,
    height: u32,
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
    let scaled_model_pos = model.position * uniforms.aspect_scale;
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
    let max_idx = arrayLength(&data_buffer) - 1u;
    let gx = clamp(u32(in.uv.x * f32(w)), 0u, w - 1u);
    let gy = clamp(u32(in.uv.y * f32(h)), 0u, h - 1u);
    let cell_index = min(gy * w + gx, max_idx);
    let val = data_buffer[cell_index];

    let eval_color = evaluate_plot_color(val, uniforms.color);
    if (eval_color.a <= 0.0) {
        discard;
    }
    return eval_color;
}

