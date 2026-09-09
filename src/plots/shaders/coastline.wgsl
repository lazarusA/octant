// Coastline overlay shader.
//
// Vertex data layout: flat array<f32> of interleaved [lon, lat] pairs.
// Segments are separated by [NaN, NaN] sentinel pairs — the vertex shader
// maps them to w=0 so the GPU clips the degenerate edge silently.
//
// Coordinate mapping:
//   Lon/lat pairs are mapped into the dataset's own geographic domain, which
//   may be [0,360] or [-180,180] for longitude and ascending or descending
//   for latitude.  Vertices outside the dataset region are discarded (w=0).

struct CoastlineUniforms {
    pan:          vec2<f32>,
    zoom:         f32,
    _pad0:        u32,
    aspect_scale: vec2<f32>,
    _pad1:        u32,
    _pad2:        u32,
    line_color:   vec4<f32>,
    // Dataset geographic bounds in degrees, always satisfying:
    //   lon_min ≤ lon_max   (west → east)
    //   lat_min ≤ lat_max   (south → north)
    // The heatmap renderer has already applied axis reorientation so that
    // NDC x=-1 ↔ lon_min, x=+1 ↔ lon_max, y=-1 ↔ lat_min, y=+1 ↔ lat_max.
    lon_min:      f32,
    lon_max:      f32,
    lat_min:      f32,   // southern edge (may be negative)
    lat_max:      f32,   // northern edge
};

@group(0) @binding(0)
var<uniform> u: CoastlineUniforms;

@group(0) @binding(1)
var<storage, read> verts: array<f32>;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0)       valid: f32,   // 0 = NaN sentinel — fragment discards
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOutput {
    var out: VertexOutput;

    let base = vid * 2u;
    var lon  = verts[base];
    let lat  = verts[base + 1u];
    let pair_base = (vid / 2u) * 4u;
    var other_lon = verts[pair_base];
    let other_lat = verts[pair_base + 1u];
    if ((vid % 2u) == 0u) {
        other_lon = verts[pair_base + 2u];
    }

    if (lon != lon || lat != lat || other_lon != other_lon || other_lat != other_lat) {
        out.pos   = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        out.valid = 0.0;
        return out;
    }

    let lon_span = u.lon_max - u.lon_min;
    let lat_span = u.lat_max - u.lat_min;

    // Guard against degenerate bounds (e.g. no dataset loaded yet).
    if (abs(lon_span) < 0.001 || abs(lat_span) < 0.001) {
        out.pos   = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        out.valid = 0.0;
        return out;
    }

    // Wrap longitude into the dataset domain when it uses a [0,360] range.
    // Natural Earth vertices are always in [-180,180], so a vertex at lon=-5
    // should become 355 when the dataset domain starts at 0.
    if (u.lon_min >= 0.0 && lon < u.lon_min - 1.0) {
        lon = lon + 360.0;
    }
    if (u.lon_min >= 0.0 && other_lon < u.lon_min - 1.0) {
        other_lon = other_lon + 360.0;
    }

    // Reject the complete pair if either endpoint is outside the domain.
    if (lon < u.lon_min - 0.1 || lon > u.lon_max + 0.1
        || lat < u.lat_min - 0.1 || lat > u.lat_max + 0.1
        || other_lon < u.lon_min - 0.1 || other_lon > u.lon_max + 0.1
        || other_lat < u.lat_min - 0.1 || other_lat > u.lat_max + 0.1
        || abs(other_lon - lon) > 180.0) {
        out.pos   = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        out.valid = 0.0;
        return out;
    }

    // Map lon → [-1, 1]:  lon_min (west) → -1,  lon_max (east) → +1.
    // Map lat → [-1, 1]:  lat_min (south) → -1, lat_max (north) → +1.
    // This matches the heatmap NDC layout after check_and_orient_axes_with_coords.
    let nx = (lon - u.lon_min) / lon_span * 2.0 - 1.0;
    let ny = (lat - u.lat_min) / lat_span * 2.0 - 1.0;

    let ndc = vec2<f32>(nx, ny) * u.aspect_scale * u.zoom + u.pan;
    out.pos   = vec4<f32>(ndc, 0.0, 1.0);
    out.valid = 1.0;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.valid < 0.5) {
        discard;
    }
    return u.line_color;
}
