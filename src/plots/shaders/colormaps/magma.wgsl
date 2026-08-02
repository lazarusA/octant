// 3: Magma scientific colormap
fn colormap_magma(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let c0 = vec3<f32>(0.000, 0.000, 0.016);
    let c1 = vec3<f32>(0.318, 0.071, 0.486);
    let c2 = vec3<f32>(0.714, 0.212, 0.475);
    let c3 = vec3<f32>(0.984, 0.533, 0.380);
    let c4 = vec3<f32>(0.988, 0.992, 0.749);
    
    if (x < 0.25)       { return mix(c0, c1, x / 0.25); }
    else if (x < 0.50)  { return mix(c1, c2, (x - 0.25) / 0.25); }
    else if (x < 0.75)  { return mix(c2, c3, (x - 0.50) / 0.25); }
    else                { return mix(c3, c4, (x - 0.75) / 0.25); }
}
