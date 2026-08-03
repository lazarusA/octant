struct Uniforms {
    rotation_y: f32,
    rotation_x: f32,
    aspect_ratio: f32,
    zoom: f32,
    displacement_strength: f32,
    sphere_mode: u32,
    width: u32,
    _pad0: u32,
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
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

fn spherical_to_cartesian(radius: f32, u: f32, v: f32) -> vec3<f32> {
    let lon = (u - 0.5) * 2.0 * 3.14159265;
    let lat = (0.5 - v) * 3.14159265;

    let cos_lat = cos(lat);
    let sin_lat = sin(lat);

    let x = radius * cos_lat * sin(lon);
    let y = radius * sin_lat;
    let z = radius * cos_lat * cos(lon);

    return vec3<f32>(x, y, z);
}

fn get_normalized_radial_dr(val: f32) -> f32 {
    let cmin = uniforms.color.cmin;
    let cmax = uniforms.color.cmax;
    let range = max(cmax - cmin, 1e-6);

    if (cmin < 0.0 && cmax > 0.0) {
        // Signed data: 0.0 is base sphere surface (radius 1.0). Positive values bulge outward (> 1.0), negative values deform inward (< 1.0 crater)
        let max_abs = max(abs(cmin), abs(cmax));
        return clamp(val / max_abs, -1.0, 1.0) * 0.4 * uniforms.displacement_strength;
    } else {
        // Unsigned data: cmin is base sphere (1.0), cmax is max radius (1.0 + dr)
        let norm_val = clamp((val - cmin) / range, 0.0, 1.0);
        return norm_val * 0.4 * uniforms.displacement_strength;
    }
}

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var out: VertexOutput;

    var raw_val: f32;
    var pos_3d: vec3<f32>;
    var normal_3d: vec3<f32>;

    if (uniforms.sphere_mode == 0u) {
        // Mode 0: Smooth Sphere Projection
        raw_val = data_buffer[model.cell_index];
        pos_3d = model.position;
        normal_3d = normalize(model.position);
    } else if (uniforms.sphere_mode == 1u) {
        // Mode 1: Smooth Terrain (Continuous deformed sphere landscape with smooth corner height interpolation & gradient normals!)
        let max_data_idx = arrayLength(&data_buffer) - 1u;
        let gx = min(model.corner_index % (uniforms.width + 1u), uniforms.width - 1u);
        let gy = min(model.corner_index / (uniforms.width + 1u), max_data_idx / uniforms.width);
        let data_idx = min(gy * uniforms.width + gx, max_data_idx);
        raw_val = data_buffer[data_idx];

        let dr = get_normalized_radial_dr(raw_val);
        pos_3d = spherical_to_cartesian(1.0 + dr, model.uv.x, model.uv.y);

        // Compute local gradient surface normal for 3D peak and in-ward valley lighting
        let val_left = data_buffer[gy * uniforms.width + max(gx, 1u) - 1u];
        let val_right = data_buffer[gy * uniforms.width + min(gx + 1u, uniforms.width - 1u)];
        let val_up = data_buffer[max(gy, 1u) - 1u * uniforms.width + gx];
        let val_down = data_buffer[min(gy + 1u, max_data_idx / uniforms.width) * uniforms.width + gx];

        let dh_du = get_normalized_radial_dr(val_right) - get_normalized_radial_dr(val_left);
        let dh_dv = get_normalized_radial_dr(val_down) - get_normalized_radial_dr(val_up);

        let base_norm = normalize(model.position);
        normal_3d = normalize(base_norm + vec3<f32>(-dh_du, 0.0, -dh_dv));
    } else if (uniforms.sphere_mode == 2u) {
        // Mode 2: Flat Steps
        raw_val = data_buffer[model.cell_index];
        let dr = get_normalized_radial_dr(raw_val);
        pos_3d = spherical_to_cartesian(1.0 + dr, model.uv.x, model.uv.y);
        normal_3d = normalize(model.position);
    } else {
        // Mode 3: 3D Radial Lego Cubes (WebGPU Instanced Unit Cube draw!)
        let grid_h = max(arrayLength(&data_buffer) / uniforms.width, 1u);
        let cell_x = instance_idx % uniforms.width;
        let cell_y = instance_idx / uniforms.width;

        let max_idx = arrayLength(&data_buffer) - 1u;
        let safe_idx = min(instance_idx, max_idx);
        raw_val = data_buffer[safe_idx];

        let dr = get_normalized_radial_dr(raw_val);

        let u = (f32(cell_x) + model.position.x) / f32(uniforms.width);
        let v = (f32(cell_y) + model.position.y) / f32(grid_h);

        var radius: f32;
        if (dr >= 0.0) {
            radius = mix(1.0, 1.0 + dr, model.position.z);
        } else {
            radius = mix(1.0 + dr, 1.0, model.position.z);
        }

        pos_3d = spherical_to_cartesian(radius, u, v);
        normal_3d = model.raw_normal;
    }

    // Rigid 3D camera rotation around Y and X axes
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

    // Rigid rotation of normal vector for 3D directional lighting
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

    // Perspective projection transformation using dynamic zoom
    let cam_dist = clamp(uniforms.zoom, 1.1, 10.0);
    let cam_z = pos_rot.z - cam_dist;
    let fov_scale = 1.6;
    let proj_x = (pos_rot.x * fov_scale) / uniforms.aspect_ratio;
    let proj_y = pos_rot.y * fov_scale;

    // Standard Depth Z mapped to 0.0..1.0
    let proj_z = (cam_z + 15.0) / 30.0;

    out.position = vec4<f32>(proj_x, proj_y, proj_z, -cam_z);
    out.uv = model.uv;
    out.val = raw_val;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let eval_color = evaluate_plot_color(in.val, uniforms.color);

    if (eval_color.a <= 0.0) {
        discard;
    }

    // 3D Directional Lighting (Light source at upper right front)
    let light_dir = normalize(vec3<f32>(0.5, 0.7, 0.9));
    let diffuse = max(dot(in.normal, light_dir), 0.25);
    let ambient = 0.35;
    let lighting = clamp(ambient + diffuse * 0.65, 0.3, 1.0);

    return vec4<f32>(eval_color.rgb * lighting, eval_color.a);
}
