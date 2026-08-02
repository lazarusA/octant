struct Uniforms {
    colormap: u32,
    rotation_y: f32,
    rotation_x: f32,
    aspect_ratio: f32,
    zoom: f32,
    displacement_strength: f32,
    surface_mode: u32,
    width: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>, // x, z, base_y_factor (1.0 = top face, 0.0 = bottom base)
    @location(1) uv: vec2<f32>,
    @location(2) cell_index: u32,
    @location(3) corner_index: u32,
    @location(4) raw_normal: vec3<f32>,
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

    // 1. Fetch scalar metric based on surface_mode (0 = Smooth Terrain, 1 = 3D Extruded Blocks)
    var raw_val: f32;
    var height: f32;
    var pos_y: f32;

    if (uniforms.surface_mode == 0u) {
        let max_data_idx = arrayLength(&data_buffer) - 1u;
        let gx = min(model.corner_index % (uniforms.width + 1u), uniforms.width - 1u);
        let gy = min(model.corner_index / (uniforms.width + 1u), max_data_idx / uniforms.width);
        let data_idx = min(gy * uniforms.width + gx, max_data_idx);
        raw_val = data_buffer[data_idx];
        let norm_val = clamp(raw_val / 100.0, 0.0, 1.0);
        height = (norm_val - 0.5) * 1.5 * uniforms.displacement_strength;
        pos_y = height;
    } else {
        raw_val = data_buffer[model.cell_index];
        let norm_val = clamp(raw_val / 100.0, 0.0, 1.0);
        height = (norm_val - 0.5) * 1.5 * uniforms.displacement_strength;
        let base_y = -0.75;
        // Interpolate between base floor and top height using model.position.z
        pos_y = mix(base_y, height, model.position.z);
    }

    let pos_3d = vec3<f32>(model.position.x, pos_y, model.position.y);

    // 2. Rotate 3D terrain/block position around Y and X camera axes
    let cy = cos(uniforms.rotation_y);
    let sy = sin(uniforms.rotation_y);
    let cx = cos(uniforms.rotation_x);
    let sx = sin(uniforms.rotation_x);

    // Y-axis rotation
    let pos_y_rot = vec3<f32>(
        cy * pos_3d.x + sy * pos_3d.z,
        pos_3d.y,
        -sy * pos_3d.x + cy * pos_3d.z
    );

    // X-axis rotation
    let pos_rot = vec3<f32>(
        pos_y_rot.x,
        cx * pos_y_rot.y - sx * pos_y_rot.z,
        sx * pos_y_rot.y + cx * pos_y_rot.z
    );

    // Rotate face normal vector for 3D directional lighting
    let norm_y_rot = vec3<f32>(
        cy * model.raw_normal.x + sy * model.raw_normal.z,
        model.raw_normal.y,
        -sy * model.raw_normal.x + cy * model.raw_normal.z
    );
    out.normal = normalize(vec3<f32>(
        norm_y_rot.x,
        cx * norm_y_rot.y - sx * norm_y_rot.z,
        sx * norm_y_rot.y + cx * norm_y_rot.z
    ));

    // Perspective projection transformation
    let cam_dist = clamp(uniforms.zoom, 1.1, 10.0);
    let cam_z = pos_rot.z - cam_dist;
    let fov_scale = 1.6;
    let proj_x = (pos_rot.x * fov_scale) / uniforms.aspect_ratio;
    let proj_y = pos_rot.y * fov_scale;

    // Depth Z
    let proj_z = (cam_z + 15.0) / 30.0;

    out.position = vec4<f32>(proj_x, proj_y, proj_z, -cam_z);
    out.uv = model.uv;
    out.val = raw_val;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let norm_val = clamp(in.val / 100.0, 0.0, 1.0);
    let base_color = sample_colormap(uniforms.colormap, norm_val);

    // 3D Directional Lighting for surface terrain & block faces
    let light_dir = normalize(vec3<f32>(0.4, 0.8, 0.6));
    let diffuse = max(dot(in.normal, light_dir), 0.25);
    let ambient = 0.35;
    let lighting = clamp(ambient + diffuse * 0.65, 0.3, 1.0);

    let final_color = base_color * lighting;
    return vec4<f32>(final_color, 1.0);
}
