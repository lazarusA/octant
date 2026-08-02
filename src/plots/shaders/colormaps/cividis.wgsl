// 6: Cividis colorblind-friendly colormap
fn colormap_cividis(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let c0 = vec3<f32>(0.000, 0.135, 0.304);
    let c1 = vec3<f32>(0.286, 0.337, 0.435);
    let c2 = vec3<f32>(0.506, 0.514, 0.463);
    let c3 = vec3<f32>(0.741, 0.702, 0.430);
    let c4 = vec3<f32>(0.996, 0.906, 0.145);

    if (x < 0.25)       { return mix(c0, c1, x / 0.25); }
    else if (x < 0.50)  { return mix(c1, c2, (x - 0.25) / 0.25); }
    else if (x < 0.75)  { return mix(c2, c3, (x - 0.50) / 0.25); }
    else                { return mix(c3, c4, (x - 0.75) / 0.25); }
}
