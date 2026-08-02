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
    @location(0) position: vec3<f32>, // x, y, z (unit cube 0..1 coordinates for instancing)
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
fn vs_main(
    model: VertexInput,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var out: VertexOutput;

    var raw_val: f32;
    var pos_3d: vec3<f32>;
    var normal_3d: vec3<f32>;

    if (uniforms.surface_mode == 0u) {
        // Mode 0: Smooth Terrain
        let max_data_idx = arrayLength(&data_buffer) - 1u;
        let gx = min(model.corner_index % (uniforms.width + 1u), uniforms.width - 1u);
        let gy = min(model.corner_index / (uniforms.width + 1u), max_data_idx / uniforms.width);
        let data_idx = min(gy * uniforms.width + gx, max_data_idx);
        raw_val = data_buffer[data_idx];

        let height = (raw_val / 100.0) * 0.8 * uniforms.displacement_strength;
        pos_3d = vec3<f32>(model.position.x, height, model.position.y);

        // Compute local gradient surface normal for 3D peak and in-ward valley lighting
        let val_left = data_buffer[gy * uniforms.width + max(gx, 1u) - 1u];
        let val_right = data_buffer[gy * uniforms.width + min(gx + 1u, uniforms.width - 1u)];
        let val_up = data_buffer[max(gy, 1u) - 1u * uniforms.width + gx];
        let val_down = data_buffer[min(gy + 1u, max_data_idx / uniforms.width) * uniforms.width + gx];

        let dh_dx = ((val_right - val_left) / 100.0) * 0.8 * uniforms.displacement_strength;
        let dh_dy = ((val_down - val_up) / 100.0) * 0.8 * uniforms.displacement_strength;

        normal_3d = normalize(vec3<f32>(-dh_dx, 1.0, -dh_dy));
    } else if (uniforms.surface_mode == 1u) {
        // Mode 1: Flat Steps (Unsigned: Extrudes upward from base 0.0)
        raw_val = data_buffer[model.cell_index];
        let norm_val = clamp(raw_val / 100.0, 0.0, 1.0);
        let height = norm_val * 0.6 * uniforms.displacement_strength;
        pos_3d = vec3<f32>(model.position.x, height, model.position.y);
        normal_3d = vec3<f32>(0.0, 1.0, 0.0);
    } else {
        // Mode 2: 3D Lego Cubes (Signed: Positive -> Upward, Negative -> In-ward/Downward)
        let grid_h = max(arrayLength(&data_buffer) / uniforms.width, 1u);
        let cell_x = instance_idx % uniforms.width;
        let cell_y = instance_idx / uniforms.width;

        let max_idx = arrayLength(&data_buffer) - 1u;
        let safe_idx = min(instance_idx, max_idx);
        raw_val = data_buffer[safe_idx];

        let height = (raw_val / 100.0) * 0.8 * uniforms.displacement_strength;

        let scale_x = 2.0 * uniforms.aspect_ratio;
        let scale_y = 2.0;

        let x0 = -uniforms.aspect_ratio + (f32(cell_x) / f32(uniforms.width)) * scale_x;
        let x1 = -uniforms.aspect_ratio + (f32(cell_x + 1u) / f32(uniforms.width)) * scale_x;

        let y0 = -1.0 + (f32(cell_y) / f32(grid_h)) * scale_y;
        let y1 = -1.0 + (f32(cell_y + 1u) / f32(grid_h)) * scale_y;

        let world_x = mix(x0, x1, model.position.x);
        let world_z = mix(y0, y1, model.position.y);

        // Positive values extrude upward from 0.0; negative values extrude downward/in-ward from 0.0
        var world_y: f32;
        if (height >= 0.0) {
            world_y = mix(0.0, height, model.position.z);
        } else {
            world_y = mix(height, 0.0, model.position.z);
        }

        pos_3d = vec3<f32>(world_x, world_y, world_z);
        normal_3d = model.raw_normal;
    }

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
        cy * normal_3d.x + sy * normal_3d.z,
        normal_3d.y,
        -sy * normal_3d.x + cy * normal_3d.z
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
    var base_color = sample_colormap(uniforms.colormap, norm_val);

    // 3D Directional Lighting for surface terrain & block faces
    let light_dir = normalize(vec3<f32>(0.4, 0.8, 0.6));
    let diffuse = max(dot(in.normal, light_dir), 0.25);
    let ambient = 0.35;
    let lighting = clamp(ambient + diffuse * 0.65, 0.3, 1.0);

    let final_color = base_color * lighting;
    return vec4<f32>(final_color, 1.0);
}
