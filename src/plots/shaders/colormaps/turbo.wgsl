// 4: Turbo rainbow colormap (Google AI)
fn colormap_turbo(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let c0 = vec3<f32>(0.190, 0.072, 0.232);
    let c1 = vec3<f32>(0.156, 0.447, 0.996);
    let c2 = vec3<f32>(0.134, 0.887, 0.525);
    let c3 = vec3<f32>(0.925, 0.875, 0.134);
    let c4 = vec3<f32>(0.900, 0.180, 0.090);

    if (x < 0.25)       { return mix(c0, c1, x / 0.25); }
    else if (x < 0.50)  { return mix(c1, c2, (x - 0.25) / 0.25); }
    else if (x < 0.75)  { return mix(c2, c3, (x - 0.50) / 0.25); }
    else                { return mix(c3, c4, (x - 0.75) / 0.25); }
}
