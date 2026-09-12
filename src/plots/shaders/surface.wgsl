struct Uniforms {
    rotation_y: f32,
    rotation_x: f32,
    aspect_ratio: f32,
    zoom: f32,
    displacement_strength: f32,
    surface_mode: u32,
    width: u32,
    height: u32,
    coord_mode: u32,
    has_reference_globe: u32,
    lon_bounds: vec2<f32>,
    lat_bounds: vec2<f32>,
    _pad: vec2<u32>,
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) raw_normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) val: f32,
    @location(2) normal: vec3<f32>,
    @location(3) world_pos: vec3<f32>,
};

fn get_normalized_height(val: f32) -> f32 {
    let cmin = uniforms.color.cmin;
    let cmax = uniforms.color.cmax;
    let range = max(cmax - cmin, 1e-6);

    if (cmin < 0.0 && cmax > 0.0) {
        let max_abs = max(abs(cmin), abs(cmax));
        return clamp(val / max_abs, -1.0, 1.0);
    } else {
        let norm_val = clamp((val - cmin) / range, 0.0, 1.0);
        return norm_val;
    }
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

    var world_x: f32;
    var world_z: f32;

    if (uniforms.coord_mode == 4u || uniforms.coord_mode == 5u) {
        let npix = max(grid_w * grid_h, 12u);
        let nside = max(u32(round(sqrt(f32(npix) / 12.0))), 1u);
        let is_nested = (uniforms.coord_mode == 5u);

        let center_coords = healpix_pixel_uv_to_lon_lat(safe_idx, vec2<f32>(0.5, 0.5), nside, is_nested);
        let corner_coords = healpix_pixel_uv_to_lon_lat(safe_idx, model.position.xy, nside, is_nested);

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

    if (uniforms.surface_mode == 0u) {
        // Mode 0: Smooth Bumpy Terrain
        if (uniforms.coord_mode == 4u || uniforms.coord_mode == 5u) {
            raw_val = data_buffer[safe_idx];
        } else {
            let corner_x = min(cell_x + u32(round(model.position.x)), grid_w - 1u);
            let corner_y = min(cell_y + u32(round(model.position.y)), grid_h - 1u);
            let corner_idx = min(corner_y * grid_w + corner_x, max_idx);
            raw_val = data_buffer[corner_idx];
        }

        let norm_h = get_normalized_height(raw_val);
        let height = norm_h * 0.8 * uniforms.displacement_strength;
        pos_3d = vec3<f32>(world_x, height, world_z);
        normal_3d = vec3<f32>(0.0, 1.0, 0.0);
    } else if (uniforms.surface_mode == 1u) {
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

    let pos_rot = rotate_camera_yx(pos_3d, uniforms.rotation_y, uniforms.rotation_x);
    let norm_rot = rotate_normal_yx(normal_3d, uniforms.rotation_y, uniforms.rotation_x);

    out.position = project_perspective(pos_rot, uniforms.aspect_ratio, uniforms.zoom, 1.6, 0.1);
    out.uv = model.uv;
    out.val = raw_val;
    out.normal = norm_rot;
    out.world_pos = pos_rot;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let eval_color = evaluate_plot_color(in.val, uniforms.color);

    if (eval_color.a < 0.01) {
        discard;
    }

    var geom_normal = in.normal;
    if (uniforms.surface_mode == 0u) {
        geom_normal = compute_screen_space_normal(in.world_pos);
    }

    let lighting = evaluate_directional_lighting(geom_normal, vec3<f32>(0.4, 0.8, 0.6), 0.35, 0.65);
    return vec4<f32>(eval_color.rgb * lighting, eval_color.a);
}
