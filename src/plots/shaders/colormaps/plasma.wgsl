// 1: Plasma scientific colormap
fn colormap_plasma(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let c0 = vec3<f32>(0.051, 0.031, 0.529);
    let c1 = vec3<f32>(0.416, 0.000, 0.659);
    let c2 = vec3<f32>(0.694, 0.165, 0.565);
    let c3 = vec3<f32>(0.882, 0.392, 0.384);
    let c4 = vec3<f32>(0.941, 0.976, 0.129);
    
    if (x < 0.25)       { return mix(c0, c1, x / 0.25); }
    else if (x < 0.50)  { return mix(c1, c2, (x - 0.25) / 0.25); }
    else if (x < 0.75)  { return mix(c2, c3, (x - 0.50) / 0.25); }
    else                { return mix(c3, c4, (x - 0.75) / 0.25); }
}
