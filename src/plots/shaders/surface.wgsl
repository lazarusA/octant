struct Uniforms {
    rotation_y: f32,
    rotation_x: f32,
    aspect_ratio: f32,
    zoom: f32,
    displacement_strength: f32,
    surface_mode: u32,
    width: u32,
    height: u32,
    coord_mode: u32,
    has_reference_globe: u32,
    lon_bounds: vec2<f32>,
    lat_bounds: vec2<f32>,
    _pad: vec2<u32>,
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

@group(0) @binding(2)
var<storage, read> coord_x_buffer: array<f32>;

@group(0) @binding(3)
var<storage, read> coord_y_buffer: array<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>, // x, y, z (unit quad or unit cube coordinates)
    @location(1) uv: vec2<f32>,
    @location(2) raw_normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) val: f32,
    @location(2) normal: vec3<f32>,
    @location(3) world_pos: vec3<f32>,
};

fn get_normalized_height(val: f32) -> f32 {
    let cmin = uniforms.color.cmin;
    let cmax = uniforms.color.cmax;
    let range = max(cmax - cmin, 1e-6);

    if (cmin < 0.0 && cmax > 0.0) {
        // Signed data: 0.0 is the base ground level. Positive values deform upward (+), negative values deform downward (-)
        let max_abs = max(abs(cmin), abs(cmax));
        return clamp(val / max_abs, -1.0, 1.0);
    } else {
        // Unsigned data: cmin is the base ground level (0.0), cmax is max height (+1.0)
        return clamp((val - cmin) / range, 0.0, 1.0);
    }
}

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let grid_w = max(uniforms.width, 1u);
    let grid_h = max(uniforms.height, 1u);

    let cell_x = instance_idx % grid_w;
    let cell_y = instance_idx / grid_w;
    let max_idx = arrayLength(&data_buffer) - 1u;
    let safe_idx = min(instance_idx, max_idx);

    // 1-to-1 exact raw pixel value (0 NaN contamination)
    var raw_val = data_buffer[safe_idx];

    let data_aspect = max(f32(grid_w) / f32(grid_h), 0.1);
    let scale_x = 2.0 * data_aspect;
    let scale_y = 2.0;

    var x0 = -data_aspect + (f32(cell_x) / f32(grid_w)) * scale_x;
    var x1 = -data_aspect + (f32(cell_x + 1u) / f32(grid_w)) * scale_x;

    var y0 = -1.0 + (f32(cell_y) / f32(grid_h)) * scale_y;
    var y1 = -1.0 + (f32(cell_y + 1u) / f32(grid_h)) * scale_y;

    if (uniforms.coord_mode == 2u) {
        let max_cx = max(arrayLength(&coord_x_buffer), 1u) - 1u;
        let cx0 = coord_x_buffer[min(cell_x, max_cx)];
        let cx1 = select(
            cx0 + (cx0 - coord_x_buffer[max(cell_x, 1u) - 1u]),
            coord_x_buffer[min(cell_x + 1u, max_cx)],
            cell_x + 1u <= max_cx
        );
        let first_x = coord_x_buffer[0];
        let last_x = coord_x_buffer[max_cx];
        let diff_x = last_x - first_x;
        let span_x = select(diff_x, 1e-6, abs(diff_x) < 1e-6);
        x0 = -data_aspect + ((cx0 - first_x) / span_x) * scale_x;
        x1 = -data_aspect + ((cx1 - first_x) / span_x) * scale_x;

        let max_cy = max(arrayLength(&coord_y_buffer), 1u) - 1u;
        let cy0 = coord_y_buffer[min(cell_y, max_cy)];
        let cy1 = select(
            cy0 + (cy0 - coord_y_buffer[max(cell_y, 1u) - 1u]),
            coord_y_buffer[min(cell_y + 1u, max_cy)],
            cell_y + 1u <= max_cy
        );
        let first_y = coord_y_buffer[0];
        let last_y = coord_y_buffer[max_cy];
        let diff_y = last_y - first_y;
        let span_y = select(diff_y, 1e-6, abs(diff_y) < 1e-6);
        y0 = -1.0 + ((cy0 - first_y) / span_y) * scale_y;
        y1 = -1.0 + ((cy1 - first_y) / span_y) * scale_y;
    }

    let world_x = mix(x0, x1, model.position.x);
    let world_z = mix(y0, y1, model.position.y);

    var pos_3d: vec3<f32>;
    var normal_3d: vec3<f32>;

    if (uniforms.surface_mode == 0u) {
        // Mode 0: Smooth Bumpy Terrain (Continuous surface mesh connecting corner vertices!)
        let corner_x = min(cell_x + u32(round(model.position.x)), grid_w - 1u);
        let corner_y = min(cell_y + u32(round(model.position.y)), grid_h - 1u);
        let corner_idx = min(corner_y * grid_w + corner_x, max_idx);
        raw_val = data_buffer[corner_idx];

        let norm_h = get_normalized_height(raw_val);
        let height = norm_h * 0.8 * uniforms.displacement_strength;
        pos_3d = vec3<f32>(world_x, height, world_z);
        normal_3d = vec3<f32>(0.0, 1.0, 0.0);
    } else if (uniforms.surface_mode == 1u) {
        // Mode 1: Flat Steps
        let norm_h = get_normalized_height(raw_val);
        let height = norm_h * 0.6 * uniforms.displacement_strength;
        pos_3d = vec3<f32>(world_x, height, world_z);
        normal_3d = vec3<f32>(0.0, 1.0, 0.0);
    } else {
        // Mode 2: 3D Lego Cubes
        let norm_h = get_normalized_height(raw_val);
        let height = norm_h * 0.8 * uniforms.displacement_strength;

        let y_base = min(0.0, height);
        let y_top = max(0.0, height);
        let world_y = mix(y_base, y_top, model.position.z);

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
    let norm_rot = normalize(vec3<f32>(
        norm_y_rot.x,
        cx * norm_y_rot.y - sx * norm_y_rot.z,
        sx * norm_y_rot.y + cx * norm_y_rot.z
    ));

    // Perspective projection transformation
    let cam_dist = clamp(uniforms.zoom, 0.1, 10.0);
    let cam_z = pos_rot.z - cam_dist;
    let dist_positive = max(-cam_z, 0.001);
    let fov_scale = 1.6;
    let proj_x = (pos_rot.x * fov_scale) / uniforms.aspect_ratio;
    let proj_y = pos_rot.y * fov_scale;

    // Linear depth projection mapped to [0.0, 1.0] for hardware depth testing
    let z_near = 0.01;
    let z_far = 50.0;
    let proj_z = (z_far / (z_far - z_near)) * dist_positive - (z_far * z_near / (z_far - z_near));

    out.position = vec4<f32>(proj_x, proj_y, proj_z, dist_positive);
    out.uv = model.uv;
    out.val = raw_val;
    out.normal = norm_rot;
    out.world_pos = pos_rot;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let eval_color = evaluate_plot_color(in.val, uniforms.color);

    if (eval_color.a < 0.01) {
        discard;
    }

    // Compute pixel-perfect surface normal from screen-space derivatives or vertex normal
    var geom_normal = in.normal;
    if (uniforms.surface_mode == 0u) {
        // Mode 0: Smooth Terrain - use screen-space partial derivatives for realistic terrain lighting (0 memory reads!)
        let dpx = dpdx(in.world_pos);
        let dpy = dpdy(in.world_pos);
        let cross_norm = cross(dpx, dpy);
        if (dot(cross_norm, cross_norm) > 1e-6) {
            geom_normal = normalize(-cross_norm);
        }
    }

    // 3D Directional Lighting for surface terrain & block faces (two-sided)
    let light_dir = normalize(vec3<f32>(0.4, 0.8, 0.6));
    let diffuse = max(abs(dot(geom_normal, light_dir)), 0.25);
    let ambient = 0.35;
    let lighting = clamp(ambient + diffuse * 0.65, 0.3, 1.0);

    return vec4<f32>(eval_color.rgb * lighting, eval_color.a);
}
