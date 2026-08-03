struct Uniforms {
    colormap: u32,
    rotation_y: f32,
    rotation_x: f32,
    aspect_x: f32,
    aspect_y: f32,
    aspect_z: f32,
    zoom: f32,
    point_size: f32,
    width: u32,
    height: u32,
    depth: u32,
    screen_aspect: f32,
    cmin: f32,
    cmax: f32,
    use_nan_color: u32,
    use_lowclip: u32,
    use_highclip: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    nan_color: vec4<f32>,
    lowclip_color: vec4<f32>,
    highclip_color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>, // Local billboard unit quad corner [-0.5, +0.5]
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) val: f32,
    @location(1) local_uv: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let max_idx = arrayLength(&data_buffer) - 1u;
    let safe_idx = min(instance_idx, max_idx);
    let raw_val = data_buffer[safe_idx];
    out.val = raw_val;
    out.local_uv = model.position;

    let grid_w = max(uniforms.width, 1u);
    let grid_h = max(uniforms.height, 1u);
    let grid_d = max(arrayLength(&data_buffer) / (grid_w * grid_h), 1u);

    // Decode 3D grid coordinates (z, y, x) from instance index
    let cell_x = instance_idx % grid_w;
    let cell_y = (instance_idx / grid_w) % grid_h;
    let cell_z = instance_idx / (grid_w * grid_h);

    // Map cell (z, y, x) to 3D world coordinates with North (+Y) at top and South (-Y) at bottom
    let norm_x = (-1.0 + (f32(cell_x) + 0.5) / f32(grid_w) * 2.0) * uniforms.aspect_x;
    let norm_y = (1.0 - (f32(cell_y) + 0.5) / f32(grid_h) * 2.0) * uniforms.aspect_y; // Inverted Y so Row 0 = North (+Y)
    let norm_z = (-1.0 + (f32(cell_z) + 0.5) / f32(grid_d) * 2.0) * uniforms.aspect_z;

    let center_3d = vec3<f32>(norm_x, norm_y, norm_z);

    // 3D Camera rotation around Y and X axes
    let cy = cos(uniforms.rotation_y);
    let sy = sin(uniforms.rotation_y);
    let cx = cos(uniforms.rotation_x);
    let sx = sin(uniforms.rotation_x);

    let center_y_rot = vec3<f32>(
        cy * center_3d.x + sy * center_3d.z,
        center_3d.y,
        -sy * center_3d.x + cy * center_3d.z
    );

    let center_rot = vec3<f32>(
        center_y_rot.x,
        cx * center_y_rot.y - sx * center_y_rot.z,
        sx * center_y_rot.y + cx * center_y_rot.z
    );

    // Offset square point billboard quad parallel to camera view plane
    let p_size = clamp(uniforms.point_size, 0.002, 0.2);
    let corner_pos = center_rot + vec3<f32>(model.position.x * p_size, model.position.y * p_size, 0.0);

    // Perspective projection transformation using dynamic zoom & screen aspect ratio
    let cam_dist = clamp(uniforms.zoom, 1.1, 10.0);
    let cam_z = corner_pos.z - cam_dist;
    let fov_scale = 1.6;
    let screen_asp = max(uniforms.screen_aspect, 0.1);
    let proj_x = (corner_pos.x * fov_scale) / screen_asp;
    let proj_y = corner_pos.y * fov_scale;

    let proj_z = (cam_z + 15.0) / 30.0;

    out.position = vec4<f32>(proj_x, proj_y, proj_z, -cam_z);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let eval_color = evaluate_plot_color(
        in.val,
        uniforms.cmin,
        uniforms.cmax,
        uniforms.colormap,
        uniforms.nan_color,
        uniforms.use_nan_color,
        uniforms.lowclip_color,
        uniforms.use_lowclip,
        uniforms.highclip_color,
        uniforms.use_highclip,
    );

    if (eval_color.a <= 0.0) {
        discard;
    }

    return eval_color;
}
