// Projection modes: maximum (1), minimum (2) and average (3) intensity.

// Opaque or premultiplied output of a projected color, per the transparency toggle.
fn projection_output(col: vec4<f32>) -> vec4<f32> {
    if (uniforms.transparency == 0u) {
        return vec4<f32>(col.rgb, 1.0);
    }
    return premultiply(col);
}

// 1. Maximum Intensity Projection (MIP) of RGB composite colors, per channel
fn mip_composite(ray: Ray) -> vec4<f32> {
    var pos = ray.start;
    var max_rgb = vec3<f32>(0.0);
    var any_hit = false;
    for (var i = 0; i < ray.count; i = i + 1) {
        let s = sample_composite(pos);
        max_rgb = max(max_rgb, s.rgb);
        any_hit = any_hit || s.a > 0.001;
        pos += ray.step;
    }
    if (!any_hit) {
        return vec4<f32>(0.0);
    }
    let alpha = select(1.0, max(max_rgb.r, max(max_rgb.g, max_rgb.b)), uniforms.transparency == 1u);
    return vec4<f32>(max_rgb * alpha, alpha);
}

// 1. Maximum Intensity Projection (MIP) with density-weighted attenuation
fn mip(ray: Ray) -> vec4<f32> {
    if (is_composite()) {
        return mip_composite(ray);
    }
    var pos = ray.start;
    var maximum: f32 = -1e30;
    var max_raw: f32 = -1e30;
    var density_sum: f32 = 0.0;
    let highclip_visible = uniforms.color.highclip_color.a > 0.0;
    let range = max(uniforms.color.cmax - uniforms.color.cmin, 0.0001);
    let inv_count = 1.0 / f32(ray.count);

    for (var i = 0; i < ray.count; i = i + 1) {
        let density = sample_scalar(pos);
        if (!is_missing(density)) {
            let norm_density = clamp((density - uniforms.color.cmin) / range, 0.0, 1.0);
            density_sum += norm_density * inv_count;
            let attenuated_density = density * exp(-uniforms.attenuation * density_sum);
            let consider_sample = (density <= uniforms.color.cmax) || highclip_visible;
            if (consider_sample && (attenuated_density > maximum)) {
                maximum = attenuated_density;
                max_raw = density;
            }
        }
        pos += ray.step;
    }
    if (max_raw == -1e30) {
        return vec4<f32>(0.0);
    }
    return projection_output(value_color(max_raw));
}

// 2. Minimum Intensity Projection (MinIP)
fn minip(ray: Ray) -> vec4<f32> {
    var pos = ray.start;
    var minimum: f32 = 1e30;
    var any_hit = false;
    let lowclip_visible = uniforms.color.lowclip_color.a > 0.0;

    for (var i = 0; i < ray.count; i = i + 1) {
        let density = sample_scalar(pos);
        let consider_sample = (density >= uniforms.color.cmin) || lowclip_visible;
        if (!is_missing(density) && consider_sample && density < minimum) {
            minimum = density;
            any_hit = true;
        }
        pos += ray.step;
    }
    if (!any_hit) {
        return vec4<f32>(0.0);
    }
    return projection_output(value_color(minimum));
}

// 3. Average / Mean Intensity Projection (Radiographic Column Transmission)
fn average_projection(ray: Ray) -> vec4<f32> {
    var pos = ray.start;
    var sum_val: f32 = 0.0;
    var valid_count: f32 = 0.0;

    for (var i = 0; i < ray.count; i = i + 1) {
        let density = sample_scalar(pos);
        if (in_display_range(density)) {
            sum_val += density;
            valid_count += 1.0;
        }
        pos += ray.step;
    }
    if (valid_count <= 0.0) {
        return vec4<f32>(0.0);
    }
    return projection_output(value_color(sum_val / valid_count));
}
