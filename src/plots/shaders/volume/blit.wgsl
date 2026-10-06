// Draws the cached volume frame (premultiplied RGBA) over the plot viewport.

@group(0) @binding(0)
var frame: texture_2d<f32>;

@group(0) @binding(1)
var frame_sampler: sampler;

struct BlitOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// One triangle covering the viewport; uv runs top-down like the frame rows.
@vertex
fn vs_blit(@builtin(vertex_index) vertex_index: u32) -> BlitOutput {
    let xy = vec2<f32>(f32((vertex_index << 1u) & 2u), f32(vertex_index & 2u));
    var out: BlitOutput;
    out.position = vec4<f32>(xy * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(xy.x, 1.0 - xy.y);
    return out;
}

@fragment
fn fs_blit(in: BlitOutput) -> @location(0) vec4<f32> {
    return textureSampleLevel(frame, frame_sampler, in.uv, 0.0);
}
