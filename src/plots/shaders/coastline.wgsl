// Coastline 2D overlay shader.
//
// Vertex data layout: flat array<f32> of explicit line-list pairs [lon0, lat0, lon1, lat1, ...].
// Coordinates are WGS-84 degrees (lon -180..180 / lat -90..90).

struct CoastlineUniforms {
    pan:            vec2<f32>,
    zoom:           f32,
    crop_to_domain: u32,
    aspect_scale:   vec2<f32>,
    _pad1:          u32,
    _pad2:          u32,
    line_color:     vec4<f32>,
    lon_min:        f32,
    lon_max:        f32,
    lat_min:        f32,
    lat_max:        f32,
};

@group(0) @binding(0)
var<uniform> u: CoastlineUniforms;

@group(0) @binding(1)
var<storage, read> verts: array<f32>;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0)       valid: f32,
    @location(1)       uv: vec2<f32>,
};

fn invalid_vertex() -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    out.valid = 0.0;
    out.uv = vec2<f32>(0.0, 0.0);
    return out;
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOutput {
    let pair_base = (vid / 2u) * 4u;
    if (pair_base + 3u >= arrayLength(&verts)) {
        return invalid_vertex();
    }

    var p0_lon = verts[pair_base];
    let p0_lat = verts[pair_base + 1u];
    var p1_lon = verts[pair_base + 2u];
    let p1_lat = verts[pair_base + 3u];

    // Reject segment if any coordinate is NaN
    if (p0_lon != p0_lon || p0_lat != p0_lat || p1_lon != p1_lon || p1_lat != p1_lat) {
        return invalid_vertex();
    }

    let lon_span = u.lon_max - u.lon_min;
    let lat_span = u.lat_max - u.lat_min;

    if (abs(lon_span) < 0.001 || abs(lat_span) < 0.001) {
        return invalid_vertex();
    }

    // Wrap longitude into [0, 360] ONLY if dataset domain genuinely extends beyond 180°
    let is_0_to_360 = (u.lon_max > 180.0);
    if (is_0_to_360) {
        if (p0_lon < 0.0) {
            p0_lon = p0_lon + 360.0;
        }
        if (p1_lon < 0.0) {
            p1_lon = p1_lon + 360.0;
        }
        if (abs(p1_lon - p0_lon) > 180.0) {
            return invalid_vertex();
        }
    } else {
        if (abs(p1_lon - p0_lon) > 180.0) {
            return invalid_vertex();
        }
    }

    // Select the current vertex in the segment pair (0 or 1)
    var cur_lon = p0_lon;
    var cur_lat = p0_lat;
    if ((vid % 2u) == 1u) {
        cur_lon = p1_lon;
        cur_lat = p1_lat;
    }

    // Normalized data domain coordinates [0, 1] across [lon_min..lon_max, lat_min..lat_max]
    let uv_x = (cur_lon - u.lon_min) / lon_span;
    let uv_y = (cur_lat - u.lat_min) / lat_span;

    // Map lon/lat to normalized device coordinates [-1, 1]
    let nx = uv_x * 2.0 - 1.0;
    let ny = uv_y * 2.0 - 1.0;

    let ndc = vec2<f32>(nx, ny) * u.aspect_scale * u.zoom + u.pan;

    var out: VertexOutput;
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.valid = 1.0;
    out.uv = vec2<f32>(uv_x, uv_y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.valid < 0.5) {
        discard;
    }
    if (u.crop_to_domain != 0u) {
        if (in.uv.x < 0.0 || in.uv.x > 1.0 || in.uv.y < 0.0 || in.uv.y > 1.0) {
            discard;
        }
    }
    return u.line_color;
}
