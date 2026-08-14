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
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> LineVertexOutput {
    var out: LineVertexOutput;
    out.cell_index = vertex_idx;
    out.line_index = instance_idx;

    let line_offset = instance_idx * uniforms.profile_length;
    let max_data_idx = arrayLength(&data_buffer) - 1u;
    let safe_idx = min(line_offset + vertex_idx, max_data_idx);
    let raw_val = data_buffer[safe_idx];
    out.raw_val = raw_val;

    let is_valid = raw_val == raw_val && abs(raw_val) < 1e30;

    let cmin = uniforms.color.cmin;
    let cmax = uniforms.color.cmax;
    let range = max(cmax - cmin, 1e-6);

    let norm_x = select(
        0.0,
        (f32(vertex_idx) / max(f32(uniforms.profile_length) - 1.0, 1.0)) * 2.0 - 1.0,
        uniforms.profile_length > 1u
    );

    let norm_y = select(-1.0, clamp(((raw_val - cmin) / range) * 2.0 - 1.0, -1.0, 1.0), is_valid);
    let pos = vec2<f32>(norm_x, norm_y);

    // Apply dynamic viewport padding: map NDC [-1.0, 1.0] within padded region
    let padded_pos = pos * (vec2<f32>(1.0, 1.0) - uniforms.viewport_padding);
    let transformed_pos = padded_pos * uniforms.zoom + uniforms.pan;

    // Hardware clipping: if vertex value is NaN or infinite, place clip_z outside [0, 1] NDC range to cull
    let clip_z = select(2.0, 0.0, is_valid);
    out.clip_position = vec4<f32>(transformed_pos, clip_z, 1.0);

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
