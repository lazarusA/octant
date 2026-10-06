// Draws the cached volume frame (premultiplied RGBA) over the plot viewport.

@group(0) @binding(0)
var frame: texture_2d<f32>;

@group(0) @binding(1)
var frame_sampler: sampler;

@fragment
fn fs_main(in: FullscreenOutput) -> @location(0) vec4<f32> {
    return textureSampleLevel(frame, frame_sampler, in.uv, 0.0);
}
