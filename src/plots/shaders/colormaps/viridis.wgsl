// 0: Viridis scientific colormap
fn colormap_viridis(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let c0 = vec3<f32>(0.267, 0.004, 0.329);
    let c1 = vec3<f32>(0.231, 0.322, 0.545);
    let c2 = vec3<f32>(0.129, 0.569, 0.551);
    let c3 = vec3<f32>(0.369, 0.788, 0.384);
    let c4 = vec3<f32>(0.992, 0.906, 0.145);
    
    if (x < 0.25)       { return mix(c0, c1, x / 0.25); }
    else if (x < 0.50)  { return mix(c1, c2, (x - 0.25) / 0.25); }
    else if (x < 0.75)  { return mix(c2, c3, (x - 0.50) / 0.25); }
    else                { return mix(c3, c4, (x - 0.75) / 0.25); }
}
