// Multi-channel composites over a heatmap's quad (`common/channel_blend.wgsl`
// blends each cell).
//
// Group 0 is the heatmap's (uniforms, data, coordinate lookup tables); only
// its leading quad uniforms (`common/quad.wgsl`) are declared, for the cell
// under each fragment. Group 1 is unused (the colormap atlas elsewhere).

struct Uniforms {
    quad: QuadUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (outside_quad(in.uv)) {
        discard;
    }
    let color = blend_cell(quad_cell(in.uv, channel_params.plane_len));
    if (color.a == 0.0) {
        if (channel_params.use_nan == 1u) {
            return channel_params.nan_color;
        }
        discard;
    }
    return color;
}
