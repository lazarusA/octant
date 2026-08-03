struct Uniforms {
    colormap: u32,
    rotation_y: f32,
    rotation_x: f32,
    aspect_x: f32,
    aspect_y: f32,
    aspect_z: f32,
    zoom: f32,
    displacement_strength: f32, // Density / Opacity scale
    step_count: u32,            // Raymarching steps (e.g. 64)
    width: u32,
    height: u32,
    depth: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Unit bounding box scaled by 3D aspect ratio [aspect_x, aspect_y, aspect_z]
    let pos_3d = vec3<f32>(
        model.position.x * uniforms.aspect_x,
        model.position.y * uniforms.aspect_y,
        model.position.z * uniforms.aspect_z
    );
    out.local_pos = model.position;

    // 3D Camera rotation around Y and X axes
    let cy = cos(uniforms.rotation_y);
    let sy = sin(uniforms.rotation_y);
    let cx = cos(uniforms.rotation_x);
    let sx = sin(uniforms.rotation_x);

    let pos_y_rot = vec3<f32>(
        cy * pos_3d.x + sy * pos_3d.z,
        pos_3d.y,
        -sy * pos_3d.x + cy * pos_3d.z
    );

    let pos_rot = vec3<f32>(
        pos_y_rot.x,
        cx * pos_y_rot.y - sx * pos_y_rot.z,
        sx * pos_y_rot.y + cx * pos_y_rot.z
    );

    // Perspective projection transformation using dynamic zoom
    let cam_dist = clamp(uniforms.zoom, 1.1, 10.0);
    let cam_z = pos_rot.z - cam_dist;
    let fov_scale = 1.6;
    let proj_x = pos_rot.x * fov_scale;
    let proj_y = pos_rot.y * fov_scale;

    let proj_z = (cam_z + 15.0) / 30.0;

    out.position = vec4<f32>(proj_x, proj_y, proj_z, -cam_z);
    return out;
}

// Ray-box intersection algorithm
fn ray_box_intersect(ray_origin: vec3<f32>, ray_dir: vec3<f32>, box_min: vec3<f32>, box_max: vec3<f32>) -> vec2<f32> {
    let inv_dir = 1.0 / ray_dir;
    let t0 = (box_min - ray_origin) * inv_dir;
    let t1 = (box_max - ray_origin) * inv_dir;

    let tmin = max(max(min(t0.x, t1.x), min(t0.y, t1.y)), min(t0.z, t1.z));
    let tmax = min(min(max(t0.x, t1.x), max(t0.y, t1.y)), max(t0.z, t1.z));

    return vec2<f32>(tmin, tmax);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Reconstruct camera ray in object space
    let cam_dist = clamp(uniforms.zoom, 1.1, 10.0);
    let ray_origin = vec3<f32>(0.0, 0.0, cam_dist);
    let ray_dir = normalize(in.local_pos - ray_origin);

    let box_min = vec3<f32>(-1.0, -1.0, -1.0);
    let box_max = vec3<f32>(1.0, 1.0, 1.0);

    let intersect = ray_box_intersect(ray_origin, ray_dir, box_min, box_max);
    let t_near = max(intersect.x, 0.0);
    let t_far = intersect.y;

    if (t_near >= t_far) {
        discard;
    }

    let steps = max(uniforms.step_count, 16u);
    let step_size = (t_far - t_near) / f32(steps);

    var accum_color = vec3<f32>(0.0);
    var accum_alpha = 0.0;

    let total_len = arrayLength(&data_buffer);
    let grid_w = max(uniforms.width, 1u);
    let grid_h = max(uniforms.height, 1u);
    let grid_d = max(uniforms.depth, 1u);

    // March ray through 3D scalar volume with solid non-transparent outer surface
    for (var i = 0u; i < steps; i = i + 1u) {
        let t = t_near + (f32(i) + 0.5) * step_size;
        let pos = ray_origin + t * ray_dir;

        // Convert [-1, 1]^3 to 3D grid cell indices (z, y, x)
        let norm_pos = pos * 0.5 + vec3<f32>(0.5);
        let gx = min(u32(norm_pos.x * f32(grid_w)), grid_w - 1u);
        let gy = min(u32((1.0 - norm_pos.y) * f32(grid_h)), grid_h - 1u);
        let gz = min(u32(norm_pos.z * f32(grid_d)), grid_d - 1u);

        let data_idx = min(gz * grid_h * grid_w + gy * grid_w + gx, total_len - 1u);
        let raw_val = data_buffer[data_idx];
        let norm_val = clamp(raw_val / 100.0, 0.0, 1.0);

        let sample_color = sample_colormap(uniforms.colormap, norm_val);
        // Solid opacity accumulation: non-zero voxels hit 1.0 opacity rapidly
        let sample_alpha = clamp(norm_val * 3.0 * uniforms.displacement_strength, 0.0, 1.0);

        // Front-to-back emission-absorption accumulation
        accum_color = accum_color + (1.0 - accum_alpha) * sample_color * sample_alpha;
        accum_alpha = accum_alpha + (1.0 - accum_alpha) * sample_alpha;

        // Early Ray Termination when surface becomes 100% solid/opaque
        if (accum_alpha >= 0.98) {
            accum_alpha = 1.0;
            break;
        }
    }

    if (accum_alpha <= 0.001) {
        discard;
    }

    return vec4<f32>(accum_color, accum_alpha);
}
