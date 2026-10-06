// Trilinear fetch through the sampler (R32Float with FLOAT32_FILTERABLE, or Rgba8Unorm).
fn fetch_trilinear(uvw: vec3<f32>) -> vec4<f32> {
    return textureSampleLevel(volume_values, volume_sampler, uvw, 0.0);
}
