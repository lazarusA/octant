// Classic surface and optical modes on nearest voxels (straight alpha):
// label surface (4), absorption (5), additive emission (6), indexed palettes (7).

// 4. Categorical / Label Segmented Surface (Binary Mask Normals)
fn label_iso(ray: Ray) -> vec4<f32> {
    for (var i = 0; i < ray.count; i = i + 1) {
        let pos = sample_pos(ray, i);
        if (exact_foreground(pos) >= 0.5) {
            // Bisection on the foreground crossing
            var lo = previous_pos(ray, i);
            var hi = pos;
            for (var step = 0; step < 3; step = step + 1) {
                let mid = 0.5 * (lo + hi);
                if (exact_foreground(mid) >= 0.5) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            let N = sobel_normal_mask(hi);
            let shaded = classic_blinnphong(N, camera_dir(ray), uniforms.light_direction, exact_rgba(pos).rgb);
            return vec4<f32>(shaded, 1.0);
        }
    }
    return vec4<f32>(0.0);
}

// 5. Optical Absorption RGBA
fn absorptionrgba(ray: Ray) -> vec4<f32> {
    var transmittance: f32 = 1.0;
    var color_sum = vec3<f32>(0.0);
    let step_size = length(ray.step);

    for (var i = 0; i < ray.count; i = i + 1) {
        let pos = sample_pos(ray, i);
        let color_sample = exact_rgba(pos);
        if (uniforms.transparency == 0u) {
            if (color_sample.a > 0.05) {
                let N = central_normal(pos);
                let shaded = classic_blinnphong(N, camera_dir(ray), uniforms.light_direction, color_sample.rgb);
                return vec4<f32>(shaded, 1.0);
            }
        } else {
            let opacity = clamp(step_size * color_sample.a * uniforms.absorption, 0.0, 1.0);
            color_sum = color_sum + (transmittance * opacity) * color_sample.rgb;
            transmittance = transmittance * (1.0 - opacity);
            if (transmittance <= 0.01) {
                break;
            }
        }
    }
    if (uniforms.transparency == 0u || 1.0 - transmittance <= 0.0) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(color_sum / (1.0 - transmittance), 1.0 - transmittance);
}

// 6. Additive RGBA (Volume Emission)
fn additivergba(ray: Ray) -> vec4<f32> {
    var integrated_color = vec4<f32>(0.0);
    let step_size = length(ray.step);

    for (var i = 0; i < ray.count; i = i + 1) {
        let density = uniforms.absorption * step_size * exact_rgba(sample_pos(ray, i));
        integrated_color = 1.0 - (1.0 - integrated_color) * (1.0 - density);
    }
    return integrated_color;
}

// 7. Volume Indexed RGBA (Palette-indexed materials; missing voxels skipped)
fn volumeindexedrgba(ray: Ray) -> vec4<f32> {
    var transmittance: f32 = 1.0;
    var color_sum = vec3<f32>(0.0);
    let step_size = length(ray.step);

    for (var i = 0; i < ray.count; i = i + 1) {
        let raw = sample_exact(sample_pos(ray, i));
        if (!is_missing(raw)) {
            let norm = clamp(f32(max(i32(raw) - 1, 0)) / 255.0, 0.0, 1.0);
            let rgb = sample_plot_colormap(uniforms.color, norm);
            let opacity = clamp(step_size * uniforms.absorption, 0.0, 1.0);
            color_sum = color_sum + (transmittance * opacity) * rgb;
            transmittance = transmittance * (1.0 - opacity);
            if (transmittance <= 0.01) {
                break;
            }
        }
    }
    if (1.0 - transmittance <= 0.0) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(color_sum / (1.0 - transmittance), 1.0 - transmittance);
}
