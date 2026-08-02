// 5: Coolwarm diverging colormap
fn colormap_coolwarm(t: f32) -> vec3<f32> {
    let x = clamp(t, 0.0, 1.0);
    let cool = vec3<f32>(0.230, 0.299, 0.754);
    let mid  = vec3<f32>(0.865, 0.865, 0.865);
    let warm = vec3<f32>(0.706, 0.016, 0.150);

    if (x < 0.50) {
        return mix(cool, mid, x / 0.50);
    } else {
        return mix(mid, warm, (x - 0.50) / 0.50);
    }
}
