// What the sphere and surface heightfields share: their uniforms, bindings,
// vertex structs, shading and fragment entry points. Each plot's file adds
// `vs_main`, `SCREEN_NORMAL_MODE` (the mode lit by screen-space normals) and
// `LIGHT_DIR`. Colors read `data_buffer`, heights `height_buffer`: the
// layer's own data, or the base layer's under an overlay draped on it.

struct Uniforms {
    camera: Camera,
    slice: Slice,
    displacement_strength: f32,
    // The plot's draw mode, numbered by each plot's own file.
    mode: u32,
    width: u32,
    height: u32,
    coord_mode: u32,
    // Bit 0: heights flipped; bit 1: heights read packed RGB composites.
    height_flags: u32,
    lon_bounds: vec2<f32>,
    lat_bounds: vec2<f32>,
    // The color range of the layer whose values make the heights.
    height_range: vec2<f32>,
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

@group(2) @binding(0)
var<storage, read> height_buffer: array<f32>;

// Height of `val`, a `height_buffer` value, before displacement scaling.
fn height_of(val: f32) -> f32 {
    return normalized_height(
        val,
        uniforms.height_range.x,
        uniforms.height_range.y,
        (uniforms.height_flags & 2u) != 0u,
        (uniforms.height_flags & 1u) != 0u,
    );
}

// The value heights read at cell `i`.
fn height_value(i: u32) -> f32 {
    return height_buffer[min(i, arrayLength(&height_buffer) - 1u)];
}

// Cell `i` of the data (`heights` false) or of the heights.
fn mesh_value(i: u32, heights: bool) -> f32 {
    if (heights) {
        return height_value(i);
    }
    return data_buffer[min(i, arrayLength(&data_buffer) - 1u)];
}

// Mean of the finite values around a HEALPix corner (`healpix_corner_cells`),
// else pixel `pix`'s own, of the data or of the heights.
fn healpix_corner_mean(pix: u32, corner_uv: vec2<f32>, nside: u32, is_nested: bool, heights: bool) -> f32 {
    let max_idx = arrayLength(&data_buffer) - 1u;
    let cells = healpix_corner_cells(pix, corner_uv, nside, is_nested, max_idx);
    var sum: f32 = 0.0;
    var count: f32 = 0.0;
    for (var k = 0u; k < 4u; k = k + 1u) {
        if (cells[k] != HEALPIX_NO_CELL) {
            let v = mesh_value(cells[k], heights);
            if (v == v && abs(v) < 1e30) {
                sum = sum + v;
                count = count + 1.0;
            }
        }
    }
    if (count > 0.0) {
        return sum / count;
    }
    return mesh_value(pix, heights);
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) raw_normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) val: f32,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    // RGB composite color, interpolated per channel: the packed value in
    // `val` must never be blended between vertices.
    @location(3) rgb: vec3<f32>,
    // Where the slicer reads the fragment: fractions of the grid's cells
    // along x and y, and its normalized height (`common/slice.wgsl`).
    @location(4) slice_at: vec3<f32>,
};

// Lit, colormapped fragment color (straight alpha).
fn shade(in: VertexOutput) -> vec4<f32> {
    if (!slice_box_keeps(uniforms.slice, in.slice_at)) {
        discard;
    }
    var eval_color = evaluate_plot_color(in.val, uniforms.color);
    if (uniforms.color.colormap == COLORMAP_RGB_COMPOSITE && eval_color.a > 0.0) {
        eval_color = vec4<f32>(in.rgb, eval_color.a);
    }

    if (eval_color.a < 0.01) {
        discard;
    }

    var geom_normal = in.normal;
    if (uniforms.mode == SCREEN_NORMAL_MODE) {
        geom_normal = compute_screen_space_normal(in.world_pos);
    }

    let lighting = evaluate_directional_lighting(geom_normal, LIGHT_DIR, 0.35, 0.65);
    return vec4<f32>(eval_color.rgb * lighting, eval_color.a);
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
