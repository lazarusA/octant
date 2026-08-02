// 2: Inferno scientific colormap
fn colormap_inferno(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let c0 = vec3<f32>(0.000, 0.000, 0.016);
    let c1 = vec3<f32>(0.341, 0.062, 0.429);
    let c2 = vec3<f32>(0.733, 0.216, 0.330);
    let c3 = vec3<f32>(0.976, 0.557, 0.035);
    let c4 = vec3<f32>(0.988, 1.000, 0.643);
    
    if (x < 0.25)       { return mix(c0, c1, x / 0.25); }
    else if (x < 0.50)  { return mix(c1, c2, (x - 0.25) / 0.25); }
    else if (x < 0.75)  { return mix(c2, c3, (x - 0.50) / 0.25); }
    else                { return mix(c3, c4, (x - 0.75) / 0.25); }
}
