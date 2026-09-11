struct Coastline3DUniforms {
    rotation_y: f32,
    rotation_x: f32,
    aspect_ratio: f32,
    zoom: f32,
    displacement_strength: f32,
    plot_kind: u32,
    plot_mode: u32,
    width: u32,
    height: u32,
    coord_mode: u32,
    crop_to_domain: u32,
    _pad_crop: u32,
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
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
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

    // Wrap longitude into [0, 360] ONLY if dataset domain genuinely extends beyond 180° (lon_bounds.y > π)
    let is_0_to_360 = (coastline_uniforms.lon_bounds.y > 3.14159265);
    if (is_0_to_360) {
        if (p0_lon_deg < 0.0) {
            p0_lon_deg = p0_lon_deg + 360.0;
        }
        if (p1_lon_deg < 0.0) {
            p1_lon_deg = p1_lon_deg + 360.0;
        }
        if (abs(p1_lon_deg - p0_lon_deg) > 180.0) {
            return invalid_vertex();
        }
    } else {
        if (abs(p1_lon_deg - p0_lon_deg) > 180.0) {
            return invalid_vertex();
        }
    }

    var cur_lon_deg = p0_lon_deg;
    var cur_lat_deg = p0_lat_deg;
    if ((vertex_index % 2u) == 1u) {
        cur_lon_deg = p1_lon_deg;
        cur_lat_deg = p1_lat_deg;
    }

    let cur_lon = cur_lon_deg * 0.0174532925;
    let cur_lat = cur_lat_deg * 0.0174532925;

    // Domain UV coordinates [0, 1] relative to dataset geographic bounds
    let uv_x = (cur_lon - coastline_uniforms.lon_bounds.x) / lon_span;
    let uv_y = (cur_lat - coastline_uniforms.lat_bounds.x) / lat_span;

    let in_bounds = (uv_x >= 0.0 && uv_x <= 1.0 && uv_y >= 0.0 && uv_y <= 1.0);

    // Height / displacement sampling from dataset buffer if within domain bounds
    var value = 0.0;
    if (in_bounds) {
        var cell_x: u32;
        var cell_y: u32;
        if (coastline_uniforms.coord_mode == 2u) {
            cell_x = find_coord_cell_x(cur_lon_deg, coastline_uniforms.width);
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
    var out: VertexOutput;
    out.position = project_perspective(rotated, coastline_uniforms.aspect_ratio, coastline_uniforms.zoom, 1.6, min_dist);
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
