// Shared geographic and spherical projection utilities for Octant WGSL shaders

fn lon_lat_to_cartesian(radius: f32, lon: f32, lat: f32) -> vec3<f32> {
    let cos_lat = cos(lat);
    let sin_lat = sin(lat);

    let x = radius * cos_lat * sin(lon);
    let y = radius * sin_lat;
    let z = radius * cos_lat * cos(lon);

    return vec3<f32>(x, y, z);
}

// ---------------------------------------------------------------------------
// Curvilinear cell-corner geometry helpers (mode 3)
// ---------------------------------------------------------------------------

/// Circular mean of two longitudes (degrees), antimeridian-safe.
fn circular_mean_lon2(a: f32, b: f32) -> f32 {
    let rad = 3.14159265 / 180.0;
    let s = sin(a * rad) + sin(b * rad);
    let c = cos(a * rad) + cos(b * rad);
    return atan2(s, c) / rad;
}

/// Circular mean of four longitudes (degrees), antimeridian-safe.
/// Matches TypeScript `getCircularMeanLongitude`.
fn circular_mean_lon4(a: f32, b: f32, c: f32, d: f32) -> f32 {
    let rad = 3.14159265 / 180.0;
    let s = sin(a * rad) + sin(b * rad) + sin(c * rad) + sin(d * rad);
    let k = cos(a * rad) + cos(b * rad) + cos(c * rad) + cos(d * rad);
    return atan2(s, k) / rad;
}

/// Converts a lon/lat pair from degrees to a unit vector on the sphere.
fn lonlat_deg_to_unit_vec(lon_deg: f32, lat_deg: f32) -> vec3<f32> {
    let lon = lon_deg * 0.01745329252;
    let lat = lat_deg * 0.01745329252;
    let cos_lat = cos(lat);
    return vec3<f32>(
        cos_lat * cos(lon),
        sin(lat),
        cos_lat * sin(lon),
    );
}

/// Returns the spherical mean of up to four cell-centre lon/lat points.
/// This is the correct pole-stable average on a sphere; simple longitude
/// averaging in degrees is undefined near ±90° latitude.
fn spherical_mean_lonlat_deg(points: array<vec2<f32>, 4>, count: u32) -> vec2<f32> {
    var sum = vec3<f32>(0.0, 0.0, 0.0);
    for (var i: u32 = 0u; i < count; i = i + 1u) {
        let p = points[i];
        let v = lonlat_deg_to_unit_vec(p.x, p.y);
        sum += v;
    }

    let norm = length(sum);
    if (norm < 1e-6) {
        return vec2<f32>(0.0, 0.0);
    }

    let n = sum / norm;
    let lon = atan2(n.z, n.x);
    let lat = atan2(n.y, sqrt(max(n.x * n.x + n.z * n.z, 0.0)));
    return vec2<f32>(lon, lat);
}

/// Computes the geographic corner (lon_rad, lat_rad) for a curvilinear cell.
///
/// Each corner is the spherical mean of the four surrounding cell centres.
/// This avoids the pole singularity of averaging longitude values directly in
/// degrees, which otherwise produces visible distortion near both poles.
///
/// corner_select:
///   0 = top-left     (row-1, col-1) × (row-1, col) × (row, col-1) × (row, col)
///   1 = top-right    (row-1, col)   × (row-1, col+1) × (row, col) × (row, col+1)
///   2 = bottom-right (row, col)     × (row, col+1) × (row+1, col) × (row+1, col+1)
///   3 = bottom-left  (row, col-1)   × (row, col) × (row+1, col-1) × (row+1, col)
///
/// flip_i = 1  → i-axis is reversed (westward), swap prev/next col
/// periodic_i  = 1 → column indices wrap around (global ocean grids)
fn curvilinear_corner_lonlat(
    cell_x: u32, cell_y: u32,
    corner_select: u32,
    grid_w: u32, grid_h: u32,
    flip_i: u32, periodic_i: u32,
) -> vec2<f32> {
    let max_idx = max(arrayLength(&coord_x_buffer), 1u) - 1u;
    let max_col = grid_w - 1u;
    let max_row = grid_h - 1u;

    // --- column neighbours with flip + periodicity ---
    var prev_col: u32;
    var next_col: u32;
    if (flip_i == 1u) {
        // i increases westward: "next" in grid-index = west = previous in lon
        if (periodic_i == 1u) {
            prev_col = (cell_x + 1u) % grid_w;
            next_col = select(cell_x - 1u, max_col, cell_x == 0u);
        } else {
            prev_col = min(cell_x + 1u, max_col);
            next_col = select(cell_x - 1u, 0u, cell_x == 0u);
        }
    } else {
        if (periodic_i == 1u) {
            prev_col = select(cell_x - 1u, max_col, cell_x == 0u);
            next_col = (cell_x + 1u) % grid_w;
        } else {
            prev_col = select(cell_x - 1u, 0u, cell_x == 0u);
            next_col = min(cell_x + 1u, max_col);
        }
    }

    // --- row neighbours (never periodic in j for ocean grids) ---
    let prev_row = select(cell_y - 1u, 0u, cell_y == 0u);
    let next_row = min(cell_y + 1u, max_row);

    // --- pick the four cell centres for this corner ---
    var r0: u32; var c0: u32;
    var r1: u32; var c1: u32;
    var r2: u32; var c2: u32;
    var r3: u32; var c3: u32;

    if (corner_select == 0u) {         // top-left
        r0 = prev_row; c0 = prev_col;
        r1 = prev_row; c1 = cell_x;
        r2 = cell_y;   c2 = prev_col;
        r3 = cell_y;   c3 = cell_x;
    } else if (corner_select == 1u) { // top-right
        r0 = prev_row; c0 = cell_x;
        r1 = prev_row; c1 = next_col;
        r2 = cell_y;   c2 = cell_x;
        r3 = cell_y;   c3 = next_col;
    } else if (corner_select == 2u) { // bottom-right
        r0 = cell_y;   c0 = cell_x;
        r1 = cell_y;   c1 = next_col;
        r2 = next_row; c2 = cell_x;
        r3 = next_row; c3 = next_col;
    } else {                           // bottom-left (3)
        r0 = cell_y;   c0 = prev_col;
        r1 = cell_y;   c1 = cell_x;
        r2 = next_row; c2 = prev_col;
        r3 = next_row; c3 = cell_x;
    }

    let i0 = min(r0 * grid_w + c0, max_idx);
    let i1 = min(r1 * grid_w + c1, max_idx);
    let i2 = min(r2 * grid_w + c2, max_idx);
    let i3 = min(r3 * grid_w + c3, max_idx);

    var corner_points: array<vec2<f32>, 4>;
    corner_points[0] = vec2<f32>(coord_x_buffer[i0], coord_y_buffer[i0]);
    corner_points[1] = vec2<f32>(coord_x_buffer[i1], coord_y_buffer[i1]);
    corner_points[2] = vec2<f32>(coord_x_buffer[i2], coord_y_buffer[i2]);
    corner_points[3] = vec2<f32>(coord_x_buffer[i3], coord_y_buffer[i3]);

    let mean = spherical_mean_lonlat_deg(corner_points, 4u);
    return mean * vec2<f32>(1.0, 1.0);
}

// ---------------------------------------------------------------------------
// Main lon/lat resolver — modes 0‒2 unchanged, mode 3 delegates to corners
// ---------------------------------------------------------------------------

fn get_lon_lat(
    cell_x: u32,
    cell_y: u32,
    model_xy: vec2<f32>,
    grid_w: u32,
    grid_h: u32,
    coord_mode: u32,
    lon_bounds: vec2<f32>,
    lat_bounds: vec2<f32>,
) -> vec2<f32> {
    if (coord_mode == 0u) {
        // Mode 0: Global Regular [-π..π] and [π/2..-π/2]
        let u = (f32(cell_x) + model_xy.x) / f32(grid_w);
        let v = (f32(cell_y) + model_xy.y) / f32(grid_h);
        let lon = (u - 0.5) * 2.0 * 3.14159265;
        let lat = (0.5 - v) * 3.14159265;
        return vec2<f32>(lon, lat);
    } else if (coord_mode == 1u) {
        // Mode 1: Regional Regular with explicit [lon_bounds, lat_bounds]
        let u = (f32(cell_x) + model_xy.x) / f32(grid_w);
        let v = (f32(cell_y) + model_xy.y) / f32(grid_h);
        let lon = mix(lon_bounds.x, lon_bounds.y, u);
        let lat = mix(lat_bounds.y, lat_bounds.x, v);
        return vec2<f32>(lon, lat);
    } else if (coord_mode == 2u) {
        // Mode 2: Irregular 1D Coordinate Buffers with heatmap-matching interval boundaries
        let bounds_u = get_cell_normalized_bounds_x(cell_x, grid_w, coord_mode);
        let bounds_v = get_cell_normalized_bounds_y(cell_y, grid_h, coord_mode);
        let u = mix(bounds_u.x, bounds_u.y, model_xy.x);
        let v = mix(bounds_v.x, bounds_v.y, model_xy.y);

        let max_cx = min(grid_w - 1u, max(arrayLength(&coord_x_buffer), 1u) - 1u);
        let first_x = coord_x_buffer[0];
        let last_x = coord_x_buffer[max_cx];
        let deg_lon = mix(first_x, last_x, u);

        let max_cy = min(grid_h - 1u, max(arrayLength(&coord_y_buffer), 1u) - 1u);
        let first_y = coord_y_buffer[0];
        let last_y = coord_y_buffer[max_cy];
        let deg_lat = mix(first_y, last_y, v);

        return vec2<f32>(deg_lon * 0.0174532925, deg_lat * 0.0174532925);
    } else {
        // Mode 3: Curvilinear 2D — cell-centre bilinear (fallback for non-sphere callers).
        // Sphere/surface shaders call curvilinear_corner_lonlat directly for vertex positions.
        let max_coord_idx = max(arrayLength(&coord_x_buffer), 1u) - 1u;
        let safe_idx = min(cell_y * grid_w + cell_x, max_coord_idx);
        let lon_deg = coord_x_buffer[safe_idx];
        let lat_deg = coord_y_buffer[safe_idx];
        return vec2<f32>(lon_deg * 0.0174532925, lat_deg * 0.0174532925);
    }
}
