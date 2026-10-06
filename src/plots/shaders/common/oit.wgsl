// Weighted blended order-independent transparency (McGuire & Bavoil 2013):
// fragment outputs of the 3D plot shaders' `fs_opaque` and `fs_oit` passes
// (`src/plots/oit/`), resolved by `shaders/oit/composite.wgsl`.

// Fragments at least this opaque are drawn by the opaque pass and write depth.
const OIT_OPAQUE_ALPHA: f32 = 0.995;

struct OitOutput {
    @location(0) accum: vec4<f32>,
    @location(1) reveal: vec4<f32>,
};

// Opaque pass: only (nearly) opaque fragments, drawn opaque.
fn oit_opaque(color: vec4<f32>) -> vec4<f32> {
    if (color.a < OIT_OPAQUE_ALPHA) {
        discard;
    }
    return vec4<f32>(color.rgb, 1.0);
}

// Translucent pass: premultiplied color times a depth weight, summed into
// `accum`; alpha into `reveal`, which blends as dst * (1 - alpha). `frag_z` is
// the fragment depth of `project_perspective` (near 0.01, far 50).
fn oit_accumulate(color: vec4<f32>, frag_z: f32) -> OitOutput {
    if (color.a >= OIT_OPAQUE_ALPHA) {
        discard;
    }
    // View distance; plots span a few units around the camera distance.
    let dist = 0.5 / (50.0 - frag_z * 49.99);
    let weight = color.a * clamp(0.03 / (1e-5 + pow(dist / 10.0, 4.0)), 1e-2, 3e3);
    var out: OitOutput;
    out.accum = vec4<f32>(color.rgb * color.a, color.a) * weight;
    out.reveal = vec4<f32>(color.a);
    return out;
}
