// The flat heightfield; `common/mesh3d.wgsl` holds what it shares with
// sphere.wgsl. Modes: 0 bumpy terrain, 1 flat steps, 2 blocks.
const SCREEN_NORMAL_MODE: u32 = 0u;
const LIGHT_DIR = vec3<f32>(0.4, 0.8, 0.6);

fn get_normalized_height(val: f32) -> f32 {
    return normalized_height(
        val,
        uniforms.color.cmin,
        uniforms.color.cmax,
        uniforms.color.colormap == COLORMAP_RGB_COMPOSITE,
        uniforms.flip_height == 1u,
    );
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
    let grid = vec2<f32>(f32(grid_w), f32(grid_h));
    var slice_xy = (vec2<f32>(f32(cell_x), f32(cell_y)) + model.position.xy) / grid;

    var world_x: f32;
    var world_z: f32;

    if (uniforms.coord_mode == 4u || uniforms.coord_mode == 5u) {
        let nside = healpix_nside(grid_w * grid_h);
        let is_nested = (uniforms.coord_mode == 5u);

        let center_coords = healpix_pixel_uv_to_lon_lat(safe_idx, vec2<f32>(0.5, 0.5), nside, is_nested);
        let healpix_uv = vec2<f32>(model.position.x, 1.0 - model.position.y);
        let corner_coords = healpix_pixel_uv_to_lon_lat(safe_idx, healpix_uv, nside, is_nested);

        let two_pi = 6.2831853;
        var d_lon = (corner_coords.x - center_coords.x) % two_pi;
        if (d_lon > 3.14159265) {
            d_lon = d_lon - two_pi;
        } else if (d_lon < -3.14159265) {
            d_lon = d_lon + two_pi;
        }
        let unwrapped_lon = center_coords.x + d_lon;
        let unwrapped_lat = corner_coords.y;

        let u = unwrapped_lon / two_pi;
        let v = 0.5 - unwrapped_lat / 3.14159265;

        world_x = (u * 2.0 - 1.0) * 2.0;
        world_z = v * 2.0 - 1.0;
        slice_xy = vec2<f32>(u, v);
    } else {
        let data_aspect = max(f32(grid_w) / f32(grid_h), 0.1);
        let scale_x = 2.0 * data_aspect;
        let scale_y = 2.0;

        let bounds_u = get_cell_normalized_bounds_x(cell_x, grid_w, uniforms.coord_mode);
        let bounds_v = get_cell_normalized_bounds_y(cell_y, grid_h, uniforms.coord_mode);

        let x0 = -data_aspect + bounds_u.x * scale_x;
        let x1 = -data_aspect + bounds_u.y * scale_x;

        let y0 = -1.0 + bounds_v.x * scale_y;
        let y1 = -1.0 + bounds_v.y * scale_y;

        world_x = mix(x0, x1, model.position.x);
        world_z = mix(y0, y1, model.position.y);
    }

    var pos_3d: vec3<f32>;
    var normal_3d: vec3<f32>;

    if (uniforms.mode == 0u) {
        // Mode 0: Smooth Bumpy Terrain
        if (uniforms.coord_mode == 4u || uniforms.coord_mode == 5u) {
            let nside = healpix_nside(grid_w * grid_h);
            let healpix_uv = vec2<f32>(model.position.x, 1.0 - model.position.y);
            raw_val = healpix_get_interpolated_corner_val(safe_idx, healpix_uv, nside, uniforms.coord_mode == 5u, max_idx);
        } else {
            let sx = smooth_corner(cell_x, model.position.x, grid_w, false);
            let sy = smooth_corner(cell_y, model.position.y, grid_h, false);
            raw_val = data_buffer[min(sy.cell * grid_w + sx.cell, max_idx)];
            let bounds_u = get_cell_normalized_bounds_x(sx.at_cell, grid_w, uniforms.coord_mode);
            let bounds_v = get_cell_normalized_bounds_y(sy.at_cell, grid_h, uniforms.coord_mode);
            let data_aspect = max(f32(grid_w) / f32(grid_h), 0.1);
            world_x = -data_aspect + mix(bounds_u.x, bounds_u.y, sx.frac) * 2.0 * data_aspect;
            world_z = -1.0 + mix(bounds_v.x, bounds_v.y, sy.frac) * 2.0;
            slice_xy = (vec2<f32>(f32(sx.at_cell), f32(sy.at_cell)) + vec2<f32>(sx.frac, sy.frac)) / grid;
        }

        let norm_h = get_normalized_height(raw_val);
        let height = norm_h * 0.8 * uniforms.displacement_strength;
        pos_3d = vec3<f32>(world_x, height, world_z);
        normal_3d = vec3<f32>(0.0, 1.0, 0.0);
    } else if (uniforms.mode == 1u) {
        // Mode 1: Flat Steps
        let norm_h = get_normalized_height(raw_val);
        let height = norm_h * 0.6 * uniforms.displacement_strength;
        pos_3d = vec3<f32>(world_x, height, world_z);
        normal_3d = vec3<f32>(0.0, 1.0, 0.0);
    } else {
        // Mode 2: 3D Lego Cubes
        let norm_h = get_normalized_height(raw_val);
        let height = norm_h * 0.8 * uniforms.displacement_strength;

        let y_base = min(0.0, height);
        let y_top = max(0.0, height);
        let world_y = mix(y_base, y_top, model.position.z);

        pos_3d = vec3<f32>(world_x, world_y, world_z);
        normal_3d = model.raw_normal;
    }

    let pos_rot = camera_view(uniforms.camera, pos_3d);
    let norm_rot = camera_view_normal(uniforms.camera, normal_3d);

    out.position = camera_project(uniforms.camera, pos_rot);
    out.val = raw_val;
    out.rgb = unpack_composite_rgb(raw_val);
    // Heights back to normalized heights (the 0.6 or 0.8 of each mode).
    let height_scale = select(0.8, 0.6, uniforms.mode == 1u) * uniforms.displacement_strength;
    out.slice_at = vec3<f32>(slice_xy, select(0.0, pos_3d.y / height_scale, height_scale > 0.0));
    if (uniforms.mode == 2u && uniforms.coord_mode != 4u && uniforms.coord_mode != 5u) {
        // Blocks are kept or cut whole, by their cell's center and value,
        // so none is left open.
        let center = (vec2<f32>(f32(cell_x), f32(cell_y)) + 0.5) / grid;
        out.slice_at = vec3<f32>(center, get_normalized_height(raw_val));
    }
    out.normal = norm_rot;
    out.world_pos = pos_rot;

    return out;
}

