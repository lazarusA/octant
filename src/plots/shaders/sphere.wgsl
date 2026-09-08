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
    curvilinear_flip_i: u32,
    curvilinear_periodic_i: u32,
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

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

fn get_normalized_radial_dr(val: f32) -> f32 {
    let cmin = uniforms.color.cmin;
    let cmax = uniforms.color.cmax;
    let range = max(cmax - cmin, 1e-6);

    if (cmin < 0.0 && cmax > 0.0) {
        let max_abs = max(abs(cmin), abs(cmax));
        return clamp(val / max_abs, -1.0, 1.0) * 0.4 * uniforms.displacement_strength;
    } else {
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

    var raw_val = data_buffer[safe_idx];

    // For curvilinear grids (mode 3) each vertex maps to an exact geographic corner.
    // The unit quad UVs encode the corner:
    //   (0,0) = top-left  → corner 0   (1,0) = top-right  → corner 1
    //   (0,1) = bot-left  → corner 3   (1,1) = bot-right  → corner 2
    var coords: vec2<f32>;
    if (uniforms.coord_mode == 3u) {
        let uv = model.uv;
        var corner_sel: u32;
        if (uv.x < 0.5 && uv.y < 0.5) {
            corner_sel = 0u; // top-left
        } else if (uv.x >= 0.5 && uv.y < 0.5) {
            corner_sel = 1u; // top-right
        } else if (uv.x >= 0.5 && uv.y >= 0.5) {
            corner_sel = 2u; // bottom-right
        } else {
            corner_sel = 3u; // bottom-left
        }
        coords = curvilinear_corner_lonlat(
            cell_x, cell_y, corner_sel,
            grid_w, grid_h,
            uniforms.curvilinear_flip_i, uniforms.curvilinear_periodic_i,
        );
    } else {
        coords = get_lon_lat(
            cell_x, cell_y, model.position.xy, grid_w, grid_h,
            uniforms.coord_mode, uniforms.lon_bounds, uniforms.lat_bounds,
        );
    }
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
        // Mode 1: Smooth Bumpy Terrain
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
        var center_coords: vec2<f32>;
        if (uniforms.coord_mode == 3u) {
            // Use the cell-centre directly for the normal anchor
            let max_coord_idx = max(arrayLength(&coord_x_buffer), 1u) - 1u;
            let ci = min(cell_y * grid_w + cell_x, max_coord_idx);
            let crad = 3.14159265 / 180.0;
            center_coords = vec2<f32>(
                coord_x_buffer[ci] * crad,
                coord_y_buffer[ci] * crad,
            );
        } else {
            center_coords = get_lon_lat(
                cell_x, cell_y, vec2<f32>(0.5, 0.5), grid_w, grid_h,
                uniforms.coord_mode, uniforms.lon_bounds, uniforms.lat_bounds,
            );
        }
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

    let pos_rot = rotate_camera_yx(pos_3d, uniforms.rotation_y, uniforms.rotation_x);
    let norm_rot = rotate_normal_yx(normal_3d, uniforms.rotation_y, uniforms.rotation_x);

    out.position = project_perspective(pos_rot, uniforms.aspect_ratio, uniforms.zoom, 1.6, 1.1);
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

    var geom_normal = in.normal;
    if (uniforms.sphere_mode == 1u) {
        geom_normal = compute_screen_space_normal(in.world_pos);
    }

    let lighting = evaluate_directional_lighting(geom_normal, vec3<f32>(0.5, 0.7, 0.9), 0.35, 0.65);
    return vec4<f32>(eval_color.rgb * lighting, eval_color.a);
}
