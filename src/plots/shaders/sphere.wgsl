struct Uniforms {
    rotation_y: f32,
    rotation_x: f32,
    aspect_ratio: f32,
    zoom: f32,
    displacement_strength: f32,
    sphere_mode: u32,
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
    @location(0) position: vec3<f32>,
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

fn lon_lat_to_cartesian(radius: f32, lon: f32, lat: f32) -> vec3<f32> {
    let cos_lat = cos(lat);
    let sin_lat = sin(lat);

    let x = radius * cos_lat * sin(lon);
    let y = radius * sin_lat;
    let z = radius * cos_lat * cos(lon);

    return vec3<f32>(x, y, z);
}

fn get_cell_coord_bounds(cell_idx: u32, len: u32, is_y: bool) -> vec2<f32> {
    if (len <= 1u) {
        return vec2<f32>(0.0, 1.0);
    }
    let max_idx = len - 1u;
    let idx = min(cell_idx, max_idx);

    var curr: f32;
    var prev: f32;
    var next: f32;

    if (is_y) {
        curr = coord_y_buffer[idx];
        prev = coord_y_buffer[max(idx, 1u) - 1u];
        next = coord_y_buffer[min(idx + 1u, max_idx)];
    } else {
        curr = coord_x_buffer[idx];
        prev = coord_x_buffer[max(idx, 1u) - 1u];
        next = coord_x_buffer[min(idx + 1u, max_idx)];
    }

    var c0: f32;
    var c1: f32;

    if (idx == 0u) {
        c0 = curr - 0.5 * (next - curr);
        c1 = 0.5 * (curr + next);
    } else if (idx == max_idx) {
        c0 = 0.5 * (prev + curr);
        c1 = curr + 0.5 * (curr - prev);
    } else {
        c0 = 0.5 * (prev + curr);
        c1 = 0.5 * (curr + next);
    }

    return vec2<f32>(c0, c1);
}

fn get_lon_lat(cell_x: u32, cell_y: u32, model_xy: vec2<f32>, grid_w: u32, grid_h: u32) -> vec2<f32> {
    if (uniforms.coord_mode == 0u) {
        // Mode 0: Global Regular [-π..π] and [π/2..-π/2]
        let u = (f32(cell_x) + model_xy.x) / f32(grid_w);
        let v = (f32(cell_y) + model_xy.y) / f32(grid_h);
        let lon = (u - 0.5) * 2.0 * 3.14159265;
        let lat = (0.5 - v) * 3.14159265;
        return vec2<f32>(lon, lat);
    } else if (uniforms.coord_mode == 1u) {
        // Mode 1: Regional Regular with explicit [lon_bounds, lat_bounds]
        let u = (f32(cell_x) + model_xy.x) / f32(grid_w);
        let v = (f32(cell_y) + model_xy.y) / f32(grid_h);
        let lon = mix(uniforms.lon_bounds.x, uniforms.lon_bounds.y, u);
        let lat = mix(uniforms.lat_bounds.y, uniforms.lat_bounds.x, v);
        return vec2<f32>(lon, lat);
    } else {
        // Mode 2: Irregular 1D Coordinate Buffers with continuous interval boundaries
        let bounds_x = get_cell_coord_bounds(cell_x, arrayLength(&coord_x_buffer), false);
        let deg_lon = mix(bounds_x.x, bounds_x.y, model_xy.x);
        let lon = deg_lon * 0.0174532925;

        let bounds_y = get_cell_coord_bounds(cell_y, arrayLength(&coord_y_buffer), true);
        let deg_lat = mix(bounds_y.x, bounds_y.y, model_xy.y);
        let lat = deg_lat * 0.0174532925;

        return vec2<f32>(lon, lat);
    }
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

    let grid_w = max(uniforms.width, 1u);
    let grid_h = max(uniforms.height, 1u);

    let cell_x = instance_idx % grid_w;
    let cell_y = instance_idx / grid_w;
    let max_idx = arrayLength(&data_buffer) - 1u;
    let safe_idx = min(instance_idx, max_idx);

    // 1-to-1 exact raw pixel value (0 NaN contamination)
    var raw_val = data_buffer[safe_idx];

    let coords = get_lon_lat(cell_x, cell_y, model.position.xy, grid_w, grid_h);
    let lon = coords.x;
    let lat = coords.y;

    var pos_3d: vec3<f32>;
    var normal_3d: vec3<f32>;

    if (uniforms.sphere_mode == 0u) {
        // Mode 0: Smooth Sphere Projection (unit sphere)
        raw_val = data_buffer[safe_idx];
        pos_3d = lon_lat_to_cartesian(1.0, lon, lat);
        normal_3d = normalize(pos_3d);
    } else if (uniforms.sphere_mode == 1u) {
        // Mode 1: Smooth Bumpy Terrain (Continuous deformed surface mesh connecting corner vertices!)
        let corner_x = min(cell_x + u32(round(model.position.x)), grid_w - 1u);
        let corner_y = min(cell_y + u32(round(model.position.y)), grid_h - 1u);
        let corner_idx = min(corner_y * grid_w + corner_x, max_idx);
        raw_val = data_buffer[corner_idx];

        let dr = get_normalized_radial_dr(raw_val);
        pos_3d = lon_lat_to_cartesian(1.0 + dr, lon, lat);
        normal_3d = normalize(pos_3d);
    } else if (uniforms.sphere_mode == 2u) {
        // Mode 2: Flat Steps
        raw_val = data_buffer[safe_idx];
        let dr = get_normalized_radial_dr(raw_val);
        let center_coords = get_lon_lat(cell_x, cell_y, vec2<f32>(0.5, 0.5), grid_w, grid_h);
        pos_3d = lon_lat_to_cartesian(1.0 + dr, lon, lat);
        normal_3d = normalize(lon_lat_to_cartesian(1.0, center_coords.x, center_coords.y));
    } else {
        // Mode 3: 3D Radial Lego Cubes
        raw_val = data_buffer[safe_idx];
        let dr = get_normalized_radial_dr(raw_val);
        var radius: f32;
        if (dr >= 0.0) {
            radius = mix(1.0, 1.0 + dr, model.position.z);
        } else {
            radius = mix(1.0 + dr, 1.0, model.position.z);
        }

        pos_3d = lon_lat_to_cartesian(radius, lon, lat);
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
    let norm_rot = normalize(vec3<f32>(
        norm_y_rot.x,
        cx * norm_y_rot.y - sx * norm_y_rot.z,
        sx * norm_y_rot.y + cx * norm_y_rot.z
    ));

    // Perspective projection transformation using dynamic zoom
    let cam_dist = clamp(uniforms.zoom, 1.1, 10.0);
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
    if (uniforms.sphere_mode == 1u) {
        // Mode 1: Smooth Terrain - use screen-space derivatives for faceted lighting (0 memory reads!)
        let dpx = dpdx(in.world_pos);
        let dpy = dpdy(in.world_pos);
        let cross_norm = cross(dpx, dpy);
        if (dot(cross_norm, cross_norm) > 1e-6) {
            geom_normal = normalize(-cross_norm);
        }
    }

    // 3D Directional Lighting with two-sided support for transparent / rotated meshes
    let light_dir = normalize(vec3<f32>(0.5, 0.7, 0.9));
    let diffuse = max(abs(dot(geom_normal, light_dir)), 0.25);
    let ambient = 0.35;
    let lighting = clamp(ambient + diffuse * 0.65, 0.3, 1.0);

    return vec4<f32>(eval_color.rgb * lighting, eval_color.a);
}
