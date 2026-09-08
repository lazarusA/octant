// Shared geographic and spherical projection utilities for Octant WGSL shaders

fn lon_lat_to_cartesian(radius: f32, lon: f32, lat: f32) -> vec3<f32> {
    let cos_lat = cos(lat);
    let sin_lat = sin(lat);

    let x = radius * cos_lat * sin(lon);
    let y = radius * sin_lat;
    let z = radius * cos_lat * cos(lon);

    return vec3<f32>(x, y, z);
}

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
        // Mode 3: Curvilinear 2D Coordinate Arrays (lon(y,x), lat(y,x))
        let max_coord_idx = max(arrayLength(&coord_x_buffer), 1u) - 1u;
        let next_x = min(cell_x + 1u, grid_w - 1u);
        let next_y = min(cell_y + 1u, grid_h - 1u);

        let idx_00 = min(cell_y * grid_w + cell_x, max_coord_idx);
        let idx_10 = min(cell_y * grid_w + next_x, max_coord_idx);
        let idx_01 = min(next_y * grid_w + cell_x, max_coord_idx);
        let idx_11 = min(next_y * grid_w + next_x, max_coord_idx);

        let fx = model_xy.x;
        let fy = model_xy.y;

        // Antimeridian-aware shortest-arc interpolation for longitude
        var dlon_0 = coord_x_buffer[idx_10] - coord_x_buffer[idx_00];
        if (dlon_0 > 180.0) { dlon_0 -= 360.0; } else if (dlon_0 < -180.0) { dlon_0 += 360.0; }
        let lon_0 = coord_x_buffer[idx_00] + dlon_0 * fx;

        var dlon_1 = coord_x_buffer[idx_11] - coord_x_buffer[idx_01];
        if (dlon_1 > 180.0) { dlon_1 -= 360.0; } else if (dlon_1 < -180.0) { dlon_1 += 360.0; }
        let lon_1 = coord_x_buffer[idx_01] + dlon_1 * fx;

        var dlon_y = lon_1 - lon_0;
        if (dlon_y > 180.0) { dlon_y -= 360.0; } else if (dlon_y < -180.0) { dlon_y += 360.0; }
        let lon_deg = lon_0 + dlon_y * fy;

        // Bilinear interpolation for latitude
        let lat_0 = mix(coord_y_buffer[idx_00], coord_y_buffer[idx_10], fx);
        let lat_1 = mix(coord_y_buffer[idx_01], coord_y_buffer[idx_11], fx);
        let lat_deg = mix(lat_0, lat_1, fy);

        return vec2<f32>(lon_deg * 0.0174532925, lat_deg * 0.0174532925);
    }
}
