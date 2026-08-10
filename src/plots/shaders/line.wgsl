struct LineVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) cell_index: u32,
    @location(2) line_index: u32,
};

struct LineVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) cell_index: u32,
    @location(1) @interpolate(flat) line_index: u32,
    @location(2) raw_val: f32,
};

struct LineUniforms {
    viewport_padding: vec2<f32>,
    line_thickness: f32,
    profile_length: u32,
    line_count: u32,
    line_mode: u32,
    pan: vec2<f32>,
    zoom: f32,
    _pad1: u32,
    color: ColorUniforms,
};

@group(0) @binding(0) var<uniform> uniforms: LineUniforms;
@group(0) @binding(1) var<storage, read> data_buffer: array<f32>;

@vertex
fn vs_main(model: LineVertexInput) -> LineVertexOutput {
    var out: LineVertexOutput;
    out.cell_index = model.cell_index;
    out.line_index = model.line_index;

    let line_offset = model.line_index * uniforms.profile_length;
    let raw_val = data_buffer[line_offset + model.cell_index];
    out.raw_val = raw_val;

    let cmin = uniforms.color.cmin;
    let cmax = uniforms.color.cmax;
    let range = max(cmax - cmin, 1e-6);

    let norm_y = select(-1.0, clamp(((raw_val - cmin) / range) * 2.0 - 1.0, -1.0, 1.0), raw_val == raw_val && abs(raw_val) < 1e30);
    let pos = vec2<f32>(model.position.x, norm_y);

    // Apply dynamic viewport padding: map NDC [-1.0, 1.0] within padded region
    let padded_pos = pos * (vec2<f32>(1.0, 1.0) - uniforms.viewport_padding);
    let transformed_pos = padded_pos * uniforms.zoom + uniforms.pan;
    out.clip_position = vec4<f32>(transformed_pos, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: LineVertexOutput) -> @location(0) vec4<f32> {
    if (uniforms.color.colormap == 999u) {
        // Flat solid line color mode (uses highclip_color as flat line color)
        return uniforms.color.highclip_color;
    }
    if (uniforms.line_mode == 1u) {
        let line_t = f32(in.line_index) / max(1.0, f32(uniforms.line_count - 1u));
        let rgb = sample_colormap(uniforms.color.colormap, line_t);
        return vec4<f32>(rgb, 1.0);
    }
    return evaluate_plot_color(in.raw_val, uniforms.color);
}
