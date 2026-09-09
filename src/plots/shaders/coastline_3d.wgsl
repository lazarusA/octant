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
    _pad_bounds: vec2<u32>,
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
    return out;
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let base = vertex_index * 2u;
    if (base + 1u >= arrayLength(&coastline_verts)) {
        return invalid_vertex();
    }

    let lon_deg = coastline_verts[base];
    let lat_deg = coastline_verts[base + 1u];
    let pair_base = (vertex_index / 2u) * 4u;
    var other_lon_deg = coastline_verts[pair_base];
    let other_lat_deg = coastline_verts[pair_base + 1u];
    if ((vertex_index % 2u) == 0u) {
        other_lon_deg = coastline_verts[pair_base + 2u];
    }
    if (lon_deg != lon_deg || lat_deg != lat_deg
        || other_lon_deg != other_lon_deg || other_lat_deg != other_lat_deg) {
        return invalid_vertex();
    }

    var lon = lon_deg * 0.0174532925;
    let lat = lat_deg * 0.0174532925;
    var other_lon = other_lon_deg * 0.0174532925;
    let lon_span = coastline_uniforms.lon_bounds.y - coastline_uniforms.lon_bounds.x;
    let lat_span = coastline_uniforms.lat_bounds.y - coastline_uniforms.lat_bounds.x;
    if (abs(lon_span) < 1e-6 || abs(lat_span) < 1e-6) {
        return invalid_vertex();
    }
    if (coastline_uniforms.lon_bounds.x >= 0.0 && lon < coastline_uniforms.lon_bounds.x) {
        lon = lon + 6.2831853;
    }
    if (coastline_uniforms.lon_bounds.x >= 0.0
        && other_lon < coastline_uniforms.lon_bounds.x) {
        other_lon = other_lon + 6.2831853;
    }
    if (lon < coastline_uniforms.lon_bounds.x || lon > coastline_uniforms.lon_bounds.y
        || lat < coastline_uniforms.lat_bounds.x || lat > coastline_uniforms.lat_bounds.y
        || other_lon < coastline_uniforms.lon_bounds.x
        || other_lon > coastline_uniforms.lon_bounds.y
        || other_lat_deg * 0.0174532925 < coastline_uniforms.lat_bounds.x
        || other_lat_deg * 0.0174532925 > coastline_uniforms.lat_bounds.y
        || abs(other_lon - lon) > 3.14159265) {
        return invalid_vertex();
    }

    let query_lon_deg = lon * 57.2957795;

    var cell_x: u32;
    var cell_y: u32;
    if (coastline_uniforms.coord_mode == 2u) {
        cell_x = find_coord_cell_x(query_lon_deg, coastline_uniforms.width);
        cell_y = find_coord_cell_y(lat_deg, coastline_uniforms.height);
    } else {
        let normalized_x = clamp((lon - coastline_uniforms.lon_bounds.x) / lon_span, 0.0, 1.0);
        let normalized_y = clamp((coastline_uniforms.lat_bounds.y - lat) / lat_span, 0.0, 1.0);
        cell_x = min(u32(normalized_x * f32(max(coastline_uniforms.width, 1u))), max(coastline_uniforms.width, 1u) - 1u);
        cell_y = min(u32(normalized_y * f32(max(coastline_uniforms.height, 1u))), max(coastline_uniforms.height, 1u) - 1u);
    }

    let data_len = arrayLength(&data_buffer);
    if (data_len == 0u) {
        return invalid_vertex();
    }
    let data_idx = min(cell_y * max(coastline_uniforms.width, 1u) + cell_x, data_len - 1u);
    let value = data_buffer[data_idx];

    var world_pos: vec3<f32>;
    if (coastline_uniforms.plot_kind == 1u) {
        var radius = 1.0;
        if (coastline_uniforms.plot_mode > 0u) {
            let dr = radial_height(value);
            radius = 1.002 + dr;
            if (coastline_uniforms.plot_mode == 3u) {
                radius = max(1.002, radius);
            }
        }
        if (coastline_uniforms.plot_mode == 0u) {
            radius = 1.002;
        }
        world_pos = lon_lat_to_cartesian(radius, lon, lat);
    } else {
        let data_aspect = max(f32(max(coastline_uniforms.width, 1u)) / f32(max(coastline_uniforms.height, 1u)), 0.1);
        let normalized_x = clamp((lon - coastline_uniforms.lon_bounds.x) / lon_span, 0.0, 1.0);
        let normalized_y = clamp((coastline_uniforms.lat_bounds.y - lat) / lat_span, 0.0, 1.0);
        let world_x = -data_aspect + normalized_x * 2.0 * data_aspect;
        let world_z = -1.0 + normalized_y * 2.0;
        var height = surface_height(value);
        if (coastline_uniforms.plot_mode == 2u) {
            height = max(0.0, height);
        }
        world_pos = vec3<f32>(world_x, height + 0.01, world_z);
    }

    let rotated = rotate_camera_yx(world_pos, coastline_uniforms.rotation_y, coastline_uniforms.rotation_x);
    var out: VertexOutput;
    out.position = project_perspective(rotated, coastline_uniforms.aspect_ratio, coastline_uniforms.zoom, 1.6, 0.1);
    out.valid = 1.0;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.valid < 0.5) {
        discard;
    }
    return coastline_uniforms.color;
}
