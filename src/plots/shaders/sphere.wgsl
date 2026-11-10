// The globe heightfield; `common/mesh3d.wgsl` holds what it shares with
// surface.wgsl. Modes: 0 smooth sphere, 1 bumpy terrain, 2 flat steps,
// 3 radial blocks.
const SCREEN_NORMAL_MODE: u32 = 1u;
const LIGHT_DIR = vec3<f32>(0.5, 0.7, 0.9);

// Radial displacement of `height_val`, a `height_buffer` value.
fn get_normalized_radial_dr(height_val: f32) -> f32 {
    let h = height_of(height_val);
    return h * 0.4 * uniforms.displacement_strength;
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
    var height_val = height_value(safe_idx);

    let coords = get_lon_lat(
        cell_x, cell_y, model.position.xy, grid_w, grid_h,
        uniforms.coord_mode, uniforms.lon_bounds, uniforms.lat_bounds,
    );
    var lon = coords.x;
    var lat = coords.y;
    let grid = vec2<f32>(f32(grid_w), f32(grid_h));
    var slice_xy = (vec2<f32>(f32(cell_x), f32(cell_y)) + model.position.xy) / grid;
    if (uniforms.coord_mode == 4u || uniforms.coord_mode == 5u) {
        // HEALPix pixels have no rows: fractions of the globe.
        slice_xy = vec2<f32>(lon / 6.2831853 + 0.5, 0.5 - lat / 3.14159265);
    }

    var pos_3d: vec3<f32>;
    var normal_3d: vec3<f32>;

    if (uniforms.mode == 0u) {
        // Mode 0: Smooth Sphere Projection (unit sphere)
        pos_3d = lon_lat_to_cartesian(1.0, lon, lat);
        normal_3d = normalize(pos_3d);
    } else if (uniforms.mode == 1u) {
        // Mode 1: Smooth Bumpy Terrain
        if (uniforms.coord_mode == 4u || uniforms.coord_mode == 5u) {
            let nside = healpix_nside(grid_w * grid_h);
            let healpix_uv = vec2<f32>(model.position.x, 1.0 - model.position.y);
            let nested = uniforms.coord_mode == 5u;
            raw_val = healpix_corner_mean(safe_idx, healpix_uv, nside, nested, false);
            height_val = healpix_corner_mean(safe_idx, healpix_uv, nside, nested, true);
        } else {
            let sx = smooth_corner(cell_x, model.position.x, grid_w, uniforms.coord_mode == 0u);
            let sy = smooth_corner(cell_y, model.position.y, grid_h, false);
            let corner_idx = min(sy.cell * grid_w + sx.cell, max_idx);
            raw_val = data_buffer[corner_idx];
            height_val = height_value(corner_idx);
            let corner = get_lon_lat(
                sx.at_cell, sy.at_cell, vec2<f32>(sx.frac, sy.frac), grid_w, grid_h,
                uniforms.coord_mode, uniforms.lon_bounds, uniforms.lat_bounds,
            );
            lon = corner.x;
            lat = corner.y;
            slice_xy = (vec2<f32>(f32(sx.at_cell), f32(sy.at_cell)) + vec2<f32>(sx.frac, sy.frac)) / grid;
        }

        let dr = get_normalized_radial_dr(height_val);
        pos_3d = lon_lat_to_cartesian(1.0 + dr, lon, lat);
        normal_3d = normalize(pos_3d);
    } else if (uniforms.mode == 2u) {
        // Mode 2: Flat Steps
        let dr = get_normalized_radial_dr(height_val);
        let center_coords = get_lon_lat(
            cell_x, cell_y, vec2<f32>(0.5, 0.5), grid_w, grid_h,
            uniforms.coord_mode, uniforms.lon_bounds, uniforms.lat_bounds,
        );
        pos_3d = lon_lat_to_cartesian(1.0 + dr, lon, lat);
        normal_3d = normalize(lon_lat_to_cartesian(1.0, center_coords.x, center_coords.y));
    } else {
        // Mode 3: 3D Radial Lego Cubes
        let dr = get_normalized_radial_dr(height_val);
        var radius: f32;
        if (dr >= 0.0) {
            radius = mix(1.0, 1.0 + dr, model.position.z);
        } else {
            radius = mix(1.0 + dr, 1.0, model.position.z);
        }

        pos_3d = lon_lat_to_cartesian(radius, lon, lat);
        normal_3d = model.raw_normal;
    }

    let pos_rot = camera_view(uniforms.camera, pos_3d);
    let norm_rot = camera_view_normal(uniforms.camera, normal_3d);

    out.position = camera_project(uniforms.camera, pos_rot);
    out.val = raw_val;
    out.rgb = unpack_composite_rgb(raw_val);
    // Radii back to normalized heights (`get_normalized_radial_dr`).
    let height_scale = 0.4 * uniforms.displacement_strength;
    let h = select(0.0, (length(pos_3d) - 1.0) / height_scale, height_scale > 0.0);
    out.slice_at = vec3<f32>(slice_xy, h);
    if (uniforms.mode == 3u && uniforms.coord_mode != 4u && uniforms.coord_mode != 5u) {
        // Blocks are kept or cut whole, by their cell's center and value,
        // so none is left open.
        let center = (vec2<f32>(f32(cell_x), f32(cell_y)) + 0.5) / grid;
        out.slice_at = vec3<f32>(center, select(0.0, get_normalized_radial_dr(height_val) / height_scale, height_scale > 0.0));
    }
    out.normal = norm_rot;
    out.world_pos = pos_rot;

    return out;
}

