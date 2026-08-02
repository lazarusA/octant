// Shared Colormaps Evaluator

fn sample_colormap(colormap_id: u32, t: f32) -> vec3<f32> {
    let norm = clamp(t, 0.0, 1.0);
    if (colormap_id == 0u) {
        return colormap_viridis(norm);
    } else if (colormap_id == 1u) {
        return colormap_plasma(norm);
    } else if (colormap_id == 2u) {
        return colormap_inferno(norm);
    } else if (colormap_id == 3u) {
        return colormap_magma(norm);
    } else if (colormap_id == 4u) {
        return colormap_turbo(norm);
    } else if (colormap_id == 5u) {
        return colormap_coolwarm(norm);
    } else if (colormap_id == 6u) {
        return colormap_cividis(norm);
    }
    return colormap_viridis(norm);
}
