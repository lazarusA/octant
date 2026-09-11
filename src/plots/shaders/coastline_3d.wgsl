struct Coastline3DUniforms {
    rotation_y: f32,
    rotation_x: f32,
    aspect_ratio: f32,
    zoom: f32,
    displacement_strength: f32,
    plot_kind: u32,
    plot_mode: u32,
    line_width: f32,
    width: u32,
    height: u32,
    coord_mode: u32,
    crop_to_domain: u32,
    lon_bounds: vec2<f32>,
    lat_bounds: vec2<f32>,
    color: vec4<f32>,
    _pad_color: vec2<u32>,
    color_range: vec2<f32>,
    _pad: vec2<u32>,
    _pad_tail: vec2<u32>,
};

@group(0) @binding(0)
var<uniform> coastline_uniforms: Coastline3DUniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

@group(0) @binding(4)
var<storage, read> coastline_verts: array<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) valid: f32,
    @location(1) uv: vec2<f32>,
};

fn normalized_height(value: f32) -> f32 {
    if (value != value) {
        return 0.0;
    }
    let cmin = coastline_uniforms.color_range.x;
    let cmax = coastline_uniforms.color_range.y;
    let range = max(cmax - cmin, 1e-6);
    if (cmin < 0.0 && cmax > 0.0) {
        let max_abs = max(abs(cmin), abs(cmax));
        return clamp(value / max_abs, -1.0, 1.0);
    }
    return clamp((value - cmin) / range, 0.0, 1.0);
}

fn surface_height(value: f32) -> f32 {
    let normalized = normalized_height(value);
    if (coastline_uniforms.plot_mode == 1u) {
        return normalized * 0.6 * coastline_uniforms.displacement_strength;
    }
    return normalized * 0.8 * coastline_uniforms.displacement_strength;
}

fn radial_height(value: f32) -> f32 {
    return normalized_height(value) * 0.4 * coastline_uniforms.displacement_strength;
}

fn invalid_vertex() -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    out.valid = 0.0;
    out.uv = vec2<f32>(0.0, 0.0);
    return out;
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let pair_base = (vertex_index / 2u) * 4u;
    if (pair_base + 3u >= arrayLength(&coastline_verts)) {
        return invalid_vertex();
    }

    var p0_lon_deg = coastline_verts[pair_base];
    let p0_lat_deg = coastline_verts[pair_base + 1u];
    var p1_lon_deg = coastline_verts[pair_base + 2u];
    let p1_lat_deg = coastline_verts[pair_base + 3u];

    if (p0_lon_deg != p0_lon_deg || p0_lat_deg != p0_lat_deg
        || p1_lon_deg != p1_lon_deg || p1_lat_deg != p1_lat_deg) {
        return invalid_vertex();
    }

    let lon_span = coastline_uniforms.lon_bounds.y - coastline_uniforms.lon_bounds.x;
    let lat_span = coastline_uniforms.lat_bounds.y - coastline_uniforms.lat_bounds.x;
    if (abs(lon_span) < 1e-6 || abs(lat_span) < 1e-6) {
        return invalid_vertex();
    }

    // Wrap longitude into [0, 360] ONLY for flat Surface mode when domain extends beyond 180°
    let is_0_to_360 = (coastline_uniforms.lon_bounds.y > 3.14159265);
    if (coastline_uniforms.plot_kind != 1u && is_0_to_360) {
        if (p0_lon_deg < 0.0) {
            p0_lon_deg = p0_lon_deg + 360.0;
        }
        if (p1_lon_deg < 0.0) {
            p1_lon_deg = p1_lon_deg + 360.0;
        }
    }
    if (abs(p1_lon_deg - p0_lon_deg) > 180.0) {
        return invalid_vertex();
    }

    let p0_lon = p0_lon_deg * 0.0174532925;
    let p0_lat = p0_lat_deg * 0.0174532925;
    let p1_lon = p1_lon_deg * 0.0174532925;
    let p1_lat = p1_lat_deg * 0.0174532925;

    var p0_uv_x = (p0_lon - coastline_uniforms.lon_bounds.x) / lon_span;
    var p1_uv_x = (p1_lon - coastline_uniforms.lon_bounds.x) / lon_span;
    if (coastline_uniforms.plot_kind == 1u && is_0_to_360) {
        let p0_adj_lon = select(p0_lon_deg, p0_lon_deg + 360.0, p0_lon_deg < 0.0) * 0.0174532925;
        let p1_adj_lon = select(p1_lon_deg, p1_lon_deg + 360.0, p1_lon_deg < 0.0) * 0.0174532925;
        p0_uv_x = (p0_adj_lon - coastline_uniforms.lon_bounds.x) / lon_span;
        p1_uv_x = (p1_adj_lon - coastline_uniforms.lon_bounds.x) / lon_span;
    }
    let p0_uv_y = (p0_lat - coastline_uniforms.lat_bounds.x) / lat_span;
    let p1_uv_y = (p1_lat - coastline_uniforms.lat_bounds.x) / lat_span;

    let is_p1 = (vertex_index % 2u) == 1u;
    var cur_lon_deg = p0_lon_deg;
    var cur_lat_deg = p0_lat_deg;
    var cur_lon = p0_lon;
    var cur_lat = p0_lat;
    var uv_x = p0_uv_x;
    var uv_y = p0_uv_y;
    if (is_p1) {
        cur_lon_deg = p1_lon_deg;
        cur_lat_deg = p1_lat_deg;
        cur_lon = p1_lon;
        cur_lat = p1_lat;
        uv_x = p1_uv_x;
        uv_y = p1_uv_y;
    }

    let in_bounds = (uv_x >= 0.0 && uv_x <= 1.0 && uv_y >= 0.0 && uv_y <= 1.0);

    // Height / displacement sampling from dataset buffer if within domain bounds
    var value = 0.0;
    if (in_bounds) {
        var cell_x: u32;
        var cell_y: u32;
        let sample_lon_deg = select(cur_lon_deg, select(cur_lon_deg, cur_lon_deg + 360.0, cur_lon_deg < 0.0), is_0_to_360);
        if (coastline_uniforms.coord_mode == 2u) {
            cell_x = find_coord_cell_x(sample_lon_deg, coastline_uniforms.width);
            cell_y = find_coord_cell_y(cur_lat_deg, coastline_uniforms.height);
        } else {
            let norm_y = clamp((coastline_uniforms.lat_bounds.y - cur_lat) / lat_span, 0.0, 1.0);
            cell_x = min(u32(clamp(uv_x, 0.0, 1.0) * f32(max(coastline_uniforms.width, 1u))), max(coastline_uniforms.width, 1u) - 1u);
            cell_y = min(u32(norm_y * f32(max(coastline_uniforms.height, 1u))), max(coastline_uniforms.height, 1u) - 1u);
        }

        let data_len = arrayLength(&data_buffer);
        if (data_len > 0u) {
            let data_idx = min(cell_y * max(coastline_uniforms.width, 1u) + cell_x, data_len - 1u);
            value = data_buffer[data_idx];
        }
    }

    var world_pos: vec3<f32>;
    var min_dist: f32 = 0.1;

    if (coastline_uniforms.plot_kind == 1u) {
        // --- Sphere Mode ---
        min_dist = 1.1;
        var radius = 1.002;
        if (coastline_uniforms.plot_mode > 0u && in_bounds) {
            let dr = radial_height(value);
            radius = 1.002 + dr;
            if (coastline_uniforms.plot_mode == 3u) {
                radius = max(1.002, radius);
            }
        }
        world_pos = lon_lat_to_cartesian(radius, cur_lon, cur_lat);
    } else {
        // --- Surface / Block Mode ---
        min_dist = 0.1;
        let data_aspect = max(f32(max(coastline_uniforms.width, 1u)) / f32(max(coastline_uniforms.height, 1u)), 0.1);
        let normalized_x = uv_x;
        let normalized_y = (coastline_uniforms.lat_bounds.y - cur_lat) / lat_span;
        let world_x = -data_aspect + normalized_x * 2.0 * data_aspect;
        let world_z = -1.0 + normalized_y * 2.0;
        var height = 0.0;
        if (in_bounds) {
            height = surface_height(value);
            if (coastline_uniforms.plot_mode == 2u) {
                height = max(0.0, height);
            }
        }
        world_pos = vec3<f32>(world_x, height + 0.003, world_z);
    }

    let rotated = rotate_camera_yx(world_pos, coastline_uniforms.rotation_y, coastline_uniforms.rotation_x);
    var proj = project_perspective(rotated, coastline_uniforms.aspect_ratio, coastline_uniforms.zoom, 1.6, min_dist);

    // Compute screen space normal from endpoints for line width
    if (coastline_uniforms.line_width > 1.0) {
        var other_world_pos: vec3<f32>;
        var other_lon = p0_lon;
        var cur_other_lat = p0_lat;
        if (!is_p1) {
            other_lon = p1_lon;
            cur_other_lat = p1_lat;
        }
        if (coastline_uniforms.plot_kind == 1u) {
            other_world_pos = lon_lat_to_cartesian(1.002, other_lon, cur_other_lat);
        } else {
            let data_aspect = max(f32(max(coastline_uniforms.width, 1u)) / f32(max(coastline_uniforms.height, 1u)), 0.1);
            let other_uv_x = select(p1_uv_x, p0_uv_x, !is_p1);
            let other_norm_y = (coastline_uniforms.lat_bounds.y - cur_other_lat) / lat_span;
            let other_x = -data_aspect + other_uv_x * 2.0 * data_aspect;
            let other_z = -1.0 + other_norm_y * 2.0;
            other_world_pos = vec3<f32>(other_x, 0.003, other_z);
        }
        let other_rotated = rotate_camera_yx(other_world_pos, coastline_uniforms.rotation_y, coastline_uniforms.rotation_x);
        let other_proj = project_perspective(other_rotated, coastline_uniforms.aspect_ratio, coastline_uniforms.zoom, 1.6, min_dist);

        let delta = (other_proj.xy / max(other_proj.w, 1e-4)) - (proj.xy / max(proj.w, 1e-4));
        let len = length(delta);
        if (len > 1e-6) {
            let dir = delta / len;
            let normal = vec2<f32>(-dir.y, dir.x);
            let num_inst = u32(clamp(round(coastline_uniforms.line_width * 2.0 - 1.0), 1.0, 7.0));
            let step_offset = f32(instance_index) - f32(num_inst - 1u) * 0.5;
            let pixel_scale = 0.0015;
            proj = vec4<f32>(
                proj.xy + normal * (step_offset * pixel_scale * proj.w),
                proj.z,
                proj.w
            );
        }
    }

    var out: VertexOutput;
    out.position = proj;
    out.valid = 1.0;
    out.uv = vec2<f32>(uv_x, uv_y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.valid < 0.5) {
        discard;
    }
    if (coastline_uniforms.crop_to_domain != 0u) {
        if (in.uv.x < 0.0 || in.uv.x > 1.0 || in.uv.y < 0.0 || in.uv.y > 1.0) {
            discard;
        }
    }
    return coastline_uniforms.color;
}
