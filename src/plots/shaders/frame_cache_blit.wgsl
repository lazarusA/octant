// Draws a mesh's or point cloud's cached frame (`plots/frame_cache.rs`)
// over the plot viewport with its depth, so what egui's pass draws after it
// (3D coastlines) is hidden behind it as if the plot were drawn there.

@group(0) @binding(0)
var frame: texture_2d<f32>;

@group(0) @binding(1)
var frame_depth: texture_depth_2d;

struct FrameOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_main(in: FullscreenOutput) -> FrameOutput {
    let size = textureDimensions(frame);
    let texel = min(vec2<u32>(in.uv * vec2<f32>(size)), size - vec2<u32>(1u));
    let color = textureLoad(frame, texel, 0);
    let depth = textureLoad(frame_depth, texel, 0);
    // Nothing drawn here: leave egui's color and depth alone.
    if (color.a == 0.0 && depth >= 1.0) {
        discard;
    }
    return FrameOutput(color, depth);
}
