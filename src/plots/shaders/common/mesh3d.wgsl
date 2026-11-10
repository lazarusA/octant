// What the sphere and surface heightfields share: their uniforms, bindings,
// vertex structs, shading and fragment entry points. Each plot's file adds
// `vs_main`, `SCREEN_NORMAL_MODE` (the mode lit by screen-space normals) and
// `LIGHT_DIR`.

struct Uniforms {
    camera: Camera,
    slice: Slice,
    displacement_strength: f32,
    // The plot's draw mode, numbered by each plot's own file.
    mode: u32,
    width: u32,
    height: u32,
    coord_mode: u32,
    flip_height: u32,
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
