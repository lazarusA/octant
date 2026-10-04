// Trilinear fetch blending eight texel loads, for devices that cannot filter
// R32Float. Addressing repeats on every axis, as the sampler does.
fn wrap_texel(i: vec3<i32>, n: vec3<i32>) -> vec3<i32> {
    return ((i % n) + n) % n;
}

fn fetch_trilinear(uvw: vec3<f32>) -> vec4<f32> {
    let n = vec3<i32>(textureDimensions(volume_values));
    let c = uvw * vec3<f32>(n) - 0.5;
    let base = floor(c);
    let f = c - base;
    let a = wrap_texel(vec3<i32>(base), n);
    let b = wrap_texel(vec3<i32>(base) + vec3<i32>(1), n);

    let c000 = textureLoad(volume_values, vec3<i32>(a.x, a.y, a.z), 0);
    let c100 = textureLoad(volume_values, vec3<i32>(b.x, a.y, a.z), 0);
    let c010 = textureLoad(volume_values, vec3<i32>(a.x, b.y, a.z), 0);
    let c110 = textureLoad(volume_values, vec3<i32>(b.x, b.y, a.z), 0);
    let c001 = textureLoad(volume_values, vec3<i32>(a.x, a.y, b.z), 0);
    let c101 = textureLoad(volume_values, vec3<i32>(b.x, a.y, b.z), 0);
    let c011 = textureLoad(volume_values, vec3<i32>(a.x, b.y, b.z), 0);
    let c111 = textureLoad(volume_values, vec3<i32>(b.x, b.y, b.z), 0);

    let y0 = mix(mix(c000, c100, f.x), mix(c010, c110, f.x), f.y);
    let y1 = mix(mix(c001, c101, f.x), mix(c011, c111, f.x), f.y);
    return mix(y0, y1, f.z);
}
