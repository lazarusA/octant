// Resolves an OIT frame over the plot viewport: the translucent layers'
// weighted average color, covering 1 - revealage, over the opaque layer.
// Output is premultiplied.

@group(0) @binding(0)
var opaque_tex: texture_2d<f32>;

@group(0) @binding(1)
var accum_tex: texture_2d<f32>;

@group(0) @binding(2)
var reveal_tex: texture_2d<f32>;

struct CompositeOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// One triangle covering the viewport; uv runs top-down like the frame rows.
@vertex
fn vs_composite(@builtin(vertex_index) vertex_index: u32) -> CompositeOutput {
    let xy = vec2<f32>(f32((vertex_index << 1u) & 2u), f32(vertex_index & 2u));
    var out: CompositeOutput;
    out.position = vec4<f32>(xy * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(xy.x, 1.0 - xy.y);
    return out;
}

@fragment
fn fs_composite(in: CompositeOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(opaque_tex));
    let p = vec2<i32>(clamp(in.uv * dims, vec2<f32>(0.0), dims - 1.0));
    let opaque = textureLoad(opaque_tex, p, 0);
    let accum = textureLoad(accum_tex, p, 0);
    let reveal = textureLoad(reveal_tex, p, 0).r;
    let coverage = 1.0 - reveal;
    let average = accum.rgb / max(accum.a, 1e-5);
    return vec4<f32>(average * coverage, coverage) + opaque * reveal;
}
