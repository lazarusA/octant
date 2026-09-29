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

struct ScatterVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) quad_uv: vec2<f32>,
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
    point_size: f32,
    use_custom_color: u32,
    show_lines: u32,
    show_points: u32,
    screen_aspect: f32,
    _pad0: vec2<u32>,
    line_color: vec4<f32>,
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
    if (uniforms.use_custom_color != 0u) {
        return uniforms.line_color;
    }
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

@vertex
fn vs_scatter(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> ScatterVertexOutput {
    var out: ScatterVertexOutput;

    let pt_idx = instance_idx % uniforms.profile_length;
    let line_idx = instance_idx / uniforms.profile_length;
    out.line_index = line_idx;

    let line_offset = line_idx * uniforms.profile_length;
    let max_data_idx = arrayLength(&data_buffer) - 1u;
    let safe_idx = min(line_offset + pt_idx, max_data_idx);
    let raw_val = data_buffer[safe_idx];
    out.raw_val = raw_val;

    let is_valid = raw_val == raw_val && abs(raw_val) < 1e30;

    let cmin = uniforms.color.cmin;
    let cmax = uniforms.color.cmax;
    let range = max(cmax - cmin, 1e-6);

    let norm_x = select(
        0.0,
        (f32(pt_idx) / max(f32(uniforms.profile_length) - 1.0, 1.0)) * 2.0 - 1.0,
        uniforms.profile_length > 1u
    );

    let norm_y = select(-1.0, clamp(((raw_val - cmin) / range) * 2.0 - 1.0, -1.0, 1.0), is_valid);
    let center_pos = vec2<f32>(norm_x, norm_y);

    let padded_pos = center_pos * (vec2<f32>(1.0, 1.0) - uniforms.viewport_padding);
    let transformed_pos = padded_pos * uniforms.zoom + uniforms.pan;

    // Corner offset for unit quad [-1, 1]
    var corner = vec2<f32>(0.0, 0.0);
    switch (vertex_idx) {
        case 0u: { corner = vec2<f32>(-1.0, -1.0); }
        case 1u: { corner = vec2<f32>( 1.0, -1.0); }
        case 2u: { corner = vec2<f32>( 1.0,  1.0); }
        case 3u: { corner = vec2<f32>(-1.0, -1.0); }
        case 4u: { corner = vec2<f32>( 1.0,  1.0); }
        case 5u: { corner = vec2<f32>(-1.0,  1.0); }
        default: { corner = vec2<f32>( 0.0,  0.0); }
    }

    out.quad_uv = corner;

    let size_scale = uniforms.point_size * 0.0025;
    let aspect_adj = vec2<f32>(size_scale / max(uniforms.screen_aspect, 0.001), size_scale);
    let final_pos = transformed_pos + corner * aspect_adj;

    let clip_z = select(2.0, 0.0, is_valid);
    out.clip_position = vec4<f32>(final_pos, clip_z, 1.0);

    return out;
}

@fragment
fn fs_scatter(in: ScatterVertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.quad_uv);
    if (dist > 1.0) {
        discard;
    }
    let alpha = 1.0 - smoothstep(0.8, 1.0, dist);

    var base_color: vec4<f32>;
    if (uniforms.use_custom_color != 0u) {
        base_color = uniforms.line_color;
    } else if (uniforms.color.colormap == 999u) {
        base_color = uniforms.color.highclip_color;
    } else if (uniforms.line_mode == 1u) {
        let line_t = f32(in.line_index) / max(1.0, f32(uniforms.line_count - 1u));
        let rgb = sample_colormap(uniforms.color.colormap, line_t);
        base_color = vec4<f32>(rgb, 1.0);
    } else {
        base_color = evaluate_plot_color(in.raw_val, uniforms.color);
    }

    return vec4<f32>(base_color.rgb, base_color.a * alpha);
}
