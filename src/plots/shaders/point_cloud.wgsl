struct Uniforms {
    camera: Camera,
    slice: Slice,
    aspect_x: f32,
    aspect_y: f32,
    aspect_z: f32,
    point_size: f32,
    width: u32,
    height: u32,
    depth: u32,
    shift_x: u32,
    shift_y: u32,
    shift_z: u32,
    scale_by_value: u32,
    _pad1: u32,
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>, // Local billboard unit quad corner [-0.5, +0.5]
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) val: f32,
    @location(1) local_uv: vec2<f32>,
};

// Size factor of a point: 1, or with Scale by value its place in the color
// range (a composite's brightness), floored at 0.1 so the smallest stay
// visible; missing values drawn with the NaN color keep full size.
fn value_scale(val: f32) -> f32 {
    if (uniforms.scale_by_value == 0u || val != val || abs(val) > 1e30) {
        return 1.0;
    }
    var t: f32;
    if (uniforms.color.colormap == COLORMAP_RGB_COMPOSITE) {
        t = length(unpack_composite_rgb(val)) / sqrt(3.0);
    } else {
        t = evaluate_scaled_norm(val, uniforms.color.cmin, uniforms.color.cmax, uniforms.color.scale_type, uniforms.color.scale_param);
    }
    return max(clamp(t, 0.0, 1.0), 0.1);
}

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let grid_w = max(uniforms.width, 1u);
    let grid_h = max(uniforms.height, 1u);
    let grid_d = max(arrayLength(&data_buffer) / (grid_w * grid_h), 1u);

    // Decode 3D grid coordinates (z, y, x) from instance index
    let cell_x = instance_idx % grid_w;
    let cell_y = (instance_idx / grid_w) % grid_h;
    let cell_z = instance_idx / (grid_w * grid_h);

    // Apply circular shift offsets along X, Y, Z
    let shifted_x = (cell_x + uniforms.shift_x) % grid_w;
    let shifted_y = (cell_y + uniforms.shift_y) % grid_h;
    let shifted_z = (cell_z + uniforms.shift_z) % grid_d;
    let max_idx = arrayLength(&data_buffer) - 1u;
    let data_idx = min(shifted_z * (grid_w * grid_h) + shifted_y * grid_w + shifted_x, max_idx);

    let raw_val = data_buffer[data_idx];
    out.val = raw_val;
    out.local_uv = model.position;

    // The slicer: points whose cell center is outside its box, or whose
    // cell no shown plane crosses.
    let grid = vec3<f32>(f32(grid_w), f32(grid_h), f32(grid_d));
    let center = (vec3<f32>(f32(cell_x), f32(cell_y), f32(cell_z)) + 0.5) / grid;
    if (!slice_box_keeps(uniforms.slice, center) || !slice_planes_keep(uniforms.slice, center, 0.5 / grid)) {
        out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return out;
    }

    // Fast-cull invisible points (NaNs, empty TrueColor background, or clipped values) in vertex shader
    let is_nan_val = raw_val != raw_val || abs(raw_val) > 1e30;
    if (is_nan_val && uniforms.color.use_nan_color == 0u) {
        out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return out;
    }
    if (uniforms.color.colormap == COLORMAP_RGB_COMPOSITE) {
        let packed = u32(raw_val);
        if ((packed & 0x00FFFFFFu) == 0u) {
            out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            return out;
        }
    } else if (!is_nan_val) {
        if (uniforms.color.use_lowclip == 0u && raw_val < uniforms.color.cmin) {
            out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            return out;
        }
        if (uniforms.color.use_highclip == 0u && raw_val > uniforms.color.cmax) {
            out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            return out;
        }
    }


    // Map cell (z, y, x) to 3D world coordinates with North (+Y) at top and South (-Y) at bottom, and latest/newest slice at front (-Z)
    let norm_x = (-0.5 + (f32(cell_x) + 0.5) / f32(grid_w)) * uniforms.aspect_x;
    let norm_y = (0.5 - (f32(cell_y) + 0.5) / f32(grid_h)) * uniforms.aspect_y; // Inverted Y so Row 0 = North (+Y)
    let norm_z = (0.5 - (f32(cell_z) + 0.5) / f32(grid_d)) * uniforms.aspect_z; // Inverted Z so newest slice is at Front (-Z)

    let center_3d = vec3<f32>(norm_x, norm_y, norm_z);

    // Offset square point billboard quad parallel to camera view plane
    // `point_size` is a fraction of the gap between points in x and y (1 = touching).
    // Z is left out: z neighbours sit behind each other, and a squashed Z would
    // shrink the points seen from above. A single row or column has no gap there.
    let gap_x = select(1e9, uniforms.aspect_x / f32(grid_w), grid_w > 1u);
    let gap_y = select(1e9, uniforms.aspect_y / f32(grid_h), grid_h > 1u);
    let gap_xy = min(gap_x, gap_y);
    let spacing = select(gap_xy, uniforms.aspect_z / f32(grid_d), gap_xy >= 1e9);
    let p_size = spacing * clamp(uniforms.point_size, 0.01, 5.0) * value_scale(raw_val);
    let center_view = camera_view(uniforms.camera, center_3d);
    let corner_pos = center_view + vec3<f32>(model.position.x * p_size, model.position.y * p_size, 0.0);
    out.position = camera_project(uniforms.camera, corner_pos);
    return out;
}

// Lit, colormapped fragment color (straight alpha).
fn shade(in: VertexOutput) -> vec4<f32> {
    if (uniforms.color.colormap == COLORMAP_RGB_COMPOSITE) {
        let packed = u32(in.val);
        if ((packed & 0x00FFFFFFu) == 0u) {
            discard;
        }
    }

    let eval_color = evaluate_plot_color(in.val, uniforms.color);

    if (eval_color.a <= 0.0) {
        discard;
    }

    return eval_color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return shade(in);
}

// Opaque and translucent passes of order-independent transparency.
@fragment
fn fs_opaque(in: VertexOutput) -> @location(0) vec4<f32> {
    return oit_opaque(shade(in));
}

@fragment
fn fs_oit(in: VertexOutput) -> OitOutput {
    return oit_accumulate(shade(in), in.position.z, uniforms.camera);
}
