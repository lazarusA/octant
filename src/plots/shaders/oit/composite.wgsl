// Resolves an OIT frame over the plot viewport: the translucent layers'
// weighted average color, covering 1 - revealage, over the opaque layer.
// Output is premultiplied.

@group(0) @binding(0)
var opaque_tex: texture_2d<f32>;

@group(0) @binding(1)
var accum_tex: texture_2d<f32>;

@group(0) @binding(2)
var reveal_tex: texture_2d<f32>;

@fragment
fn fs_main(in: FullscreenOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(opaque_tex));
    let p = vec2<i32>(clamp(in.uv * dims, vec2<f32>(0.0), dims - 1.0));
    let opaque = textureLoad(opaque_tex, p, 0);
    // An overflowed half-float sum (infinity) is clamped so the average stays finite.
    let accum = min(textureLoad(accum_tex, p, 0), vec4<f32>(65504.0));
    let reveal = textureLoad(reveal_tex, p, 0).r;
    let coverage = 1.0 - reveal;
    let average = accum.rgb / max(accum.a, 1e-5);
    return vec4<f32>(average * coverage, coverage) + opaque * reveal;
}
