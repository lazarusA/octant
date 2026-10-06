// Projection modes: maximum (1), minimum (2) and average (3) intensity, on
// nearest voxels as the original shader drew them (straight alpha).

// Opaque or translucent output of a projected color, per the transparency toggle.
fn projection_output(col: vec4<f32>) -> vec4<f32> {
    if (uniforms.transparency == 0u) {
        return vec4<f32>(col.rgb, 1.0);
    }
    return col;
}

// 1. Maximum Intensity Projection (MIP) of RGB composite colors, per channel
fn mip_composite(ray: Ray) -> vec4<f32> {
    var max_rgb = vec3<f32>(0.0);
    var any_hit = false;
    for (var i = 0; i < ray.count; i = i + 1) {
        let t = exact_texel(sample_pos(ray, i));
        if (!texel_missing(t)) {
            let c = textureLoad(volume_values, t, 0);
            max_rgb = max(max_rgb, c.rgb);
            any_hit = any_hit || c.a > 0.001;
        }
    }
    if (!any_hit) {
        return vec4<f32>(0.0);
    }
    let alpha = select(1.0, max(max_rgb.r, max(max_rgb.g, max_rgb.b)), uniforms.transparency == 1u);
    return vec4<f32>(max_rgb, alpha);
}

// 1. Maximum Intensity Projection (MIP) with density-weighted attenuation
fn mip(ray: Ray) -> vec4<f32> {
    if (is_composite()) {
        return mip_composite(ray);
    }
    var maximum: f32 = -1e30;
    var max_raw: f32 = -1e30;
    var density_sum: f32 = 0.0;
    let highclip_visible = uniforms.color.highclip_color.a > 0.0;
    let range = max(uniforms.color.cmax - uniforms.color.cmin, 0.0001);

    for (var i = 0; i < ray.count; i = i + 1) {
        let density = sample_exact(sample_pos(ray, i));
        if (!is_missing(density)) {
            let norm_density = clamp((density - uniforms.color.cmin) / range, 0.0, 1.0);
            density_sum += norm_density / f32(ray.count);
            let attenuated_density = density * exp(-uniforms.attenuation * density_sum);
            let consider_sample = (density <= uniforms.color.cmax) || highclip_visible;
            if (consider_sample && (attenuated_density > maximum)) {
                maximum = attenuated_density;
                max_raw = density;
            }
        }
    }
    if (max_raw == -1e30) {
        return vec4<f32>(0.0);
    }
    return projection_output(evaluate_plot_color(max_raw, uniforms.color));
}

// 2. Minimum Intensity Projection (MinIP)
fn minip(ray: Ray) -> vec4<f32> {
    var minimum: f32 = 1e30;
    var any_hit = false;
    let lowclip_visible = uniforms.color.lowclip_color.a > 0.0;

    for (var i = 0; i < ray.count; i = i + 1) {
        let density = sample_exact(sample_pos(ray, i));
        if (!is_missing(density)) {
            let consider_sample = (density >= uniforms.color.cmin) || lowclip_visible;
            if (consider_sample && (density < minimum)) {
                minimum = density;
                any_hit = true;
            }
        }
    }
    if (!any_hit) {
        return vec4<f32>(0.0);
    }
    return projection_output(evaluate_plot_color(minimum, uniforms.color));
}

// 3. Average / Mean Intensity Projection (Radiographic Column Transmission)
fn average_projection(ray: Ray) -> vec4<f32> {
    var sum_val: f32 = 0.0;
    var valid_count: f32 = 0.0;

    for (var i = 0; i < ray.count; i = i + 1) {
        let density = sample_exact(sample_pos(ray, i));
        if (!is_missing(density) && density >= uniforms.color.cmin && density <= uniforms.color.cmax) {
            sum_val += density;
            valid_count += 1.0;
        }
    }
    if (valid_count <= 0.0) {
        return vec4<f32>(0.0);
    }
    return projection_output(evaluate_plot_color(sum_val / valid_count, uniforms.color));
}
