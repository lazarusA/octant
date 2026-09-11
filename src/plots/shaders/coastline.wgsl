// Coastline 2D overlay shader.
//
// Vertex data layout: flat array<f32> of explicit line-list pairs [lon0, lat0, lon1, lat1, ...].
// Coordinates are WGS-84 degrees (lon -180..180 / lat -90..90).

struct CoastlineUniforms {
    pan:            vec2<f32>,
    zoom:           f32,
    crop_to_domain: u32,
    aspect_scale:   vec2<f32>,
    line_width:     f32,
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
fn vs_main(
    @builtin(vertex_index) vid: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
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

    let p0_uv_x = (p0_lon - u.lon_min) / lon_span;
    let p0_uv_y = (p0_lat - u.lat_min) / lat_span;
    let p0_ndc = vec2<f32>(p0_uv_x * 2.0 - 1.0, p0_uv_y * 2.0 - 1.0) * u.aspect_scale * u.zoom + u.pan;

    let p1_uv_x = (p1_lon - u.lon_min) / lon_span;
    let p1_uv_y = (p1_lat - u.lat_min) / lat_span;
    let p1_ndc = vec2<f32>(p1_uv_x * 2.0 - 1.0, p1_uv_y * 2.0 - 1.0) * u.aspect_scale * u.zoom + u.pan;

    var cur_ndc = p0_ndc;
    var uv_x = p0_uv_x;
    var uv_y = p0_uv_y;
    if ((vid % 2u) == 1u) {
        cur_ndc = p1_ndc;
        uv_x = p1_uv_x;
        uv_y = p1_uv_y;
    }

    // Apply screen-space line thickness offset for multi-instance passes
    let delta = p1_ndc - p0_ndc;
    let len = length(delta);
    if (len > 1e-6 && u.line_width > 1.0) {
        let dir = delta / len;
        let normal = vec2<f32>(-dir.y, dir.x);
        let num_inst = u32(clamp(round(u.line_width * 2.0 - 1.0), 1.0, 7.0));
        let step_offset = f32(instance_index) - f32(num_inst - 1u) * 0.5;
        let pixel_scale = 0.0015;
        cur_ndc = cur_ndc + normal * (step_offset * pixel_scale);
    }

    var out: VertexOutput;
    out.pos = vec4<f32>(cur_ndc, 0.0, 1.0);
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
