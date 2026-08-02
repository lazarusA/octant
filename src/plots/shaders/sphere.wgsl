struct Uniforms {
    colormap: u32,
    rotation_y: f32,
    rotation_x: f32,
    aspect_ratio: f32,
    zoom: f32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) cell_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) val: f32,
    @location(2) normal: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // 1. Rotate 3D sphere vertex position around Y and X axes
    let cy = cos(uniforms.rotation_y);
    let sy = sin(uniforms.rotation_y);
    let cx = cos(uniforms.rotation_x);
    let sx = sin(uniforms.rotation_x);

    // Y-axis rotation
    let pos_y_rot = vec3<f32>(
        cy * model.position.x + sy * model.position.z,
        model.position.y,
        -sy * model.position.x + cy * model.position.z
    );

    // X-axis rotation
    let pos_rot = vec3<f32>(
        pos_y_rot.x,
        cx * pos_y_rot.y - sx * pos_y_rot.z,
        sx * pos_y_rot.y + cx * pos_y_rot.z
    );

    // Normal vector after rotation (since sphere is centered at origin)
    out.normal = normalize(pos_rot);

    // Perspective projection transformation using dynamic zoom (uniforms.zoom)
    let cam_dist = clamp(uniforms.zoom, 1.1, 10.0);
    let cam_z = pos_rot.z - cam_dist;
    let fov_scale = 1.6;
    let proj_x = (pos_rot.x * fov_scale) / uniforms.aspect_ratio;
    let proj_y = pos_rot.y * fov_scale;

    // Standard Depth Z mapped to 0.0..1.0
    let proj_z = (cam_z + 15.0) / 30.0;

    out.position = vec4<f32>(proj_x, proj_y, proj_z, -cam_z);
    out.uv = model.uv;
    out.val = data_buffer[model.cell_index];

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let norm_val = clamp(in.val / 100.0, 0.0, 1.0);
    let base_color = sample_colormap(uniforms.colormap, norm_val);

    // 3D Directional Lighting (Light source at upper right front)
    let light_dir = normalize(vec3<f32>(0.5, 0.7, 0.9));
    let diffuse = max(dot(in.normal, light_dir), 0.25);
    let ambient = 0.35;
    let lighting = clamp(ambient + diffuse * 0.65, 0.3, 1.0);

    let final_color = base_color * lighting;
    return vec4<f32>(final_color, 1.0);
}
