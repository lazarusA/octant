// Direct volume rendering (mode 0) and the categorical label surface (mode 4).

fn in_display_range(d: f32) -> bool {
    return !is_missing(d) && d >= uniforms.color.cmin && d <= uniforms.color.cmax;
}

// Straight RGBA of a DVR sample: transfer color and opacity in range, enabled
// NaN or clip colors outside it, transparent otherwise.
fn dvr_sample(d: f32) -> vec4<f32> {
    let c = uniforms.color;
    if (is_missing(d)) {
        return select(vec4<f32>(0.0), c.nan_color, c.use_nan_color == 1u);
    }
    if (d < c.cmin) {
        return select(vec4<f32>(0.0), c.lowclip_color, c.use_lowclip == 1u);
    }
    if (d > c.cmax) {
        return select(vec4<f32>(0.0), c.highclip_color, c.use_highclip == 1u);
    }
    return transfer_at(scale_position(d));
}

// Bisects between `outside` and `inside` (the samples around the first
// in-range crossing) to place an opaque surface between steps.
fn refine_range_hit(outside: vec3<f32>, inside: vec3<f32>) -> vec3<f32> {
    var lo = outside;
    var hi = inside;
    for (var i = 0; i < 6; i = i + 1) {
        let mid = 0.5 * (lo + hi);
        if (in_display_range(sample_scalar(mid))) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    return hi;
}

// 0. Direct Volume Rendering (DVR) with front-to-back alpha compositing
fn volume_dvr(ray: Ray) -> vec4<f32> {
    if (is_composite()) {
        return dvr_composite(ray);
    }
    var pos = ray.start;
    var prev = ray.start - ray.step;
    var accum = vec3<f32>(0.0);
    var alpha_acc: f32 = 0.0;

    for (var i = 0; i < ray.count; i = i + 1) {
        let d = sample_scalar(pos);
        if (uniforms.transparency == 0u && in_display_range(d)) {
            let hit = refine_range_hit(prev, pos);
            return shade_surface(hit, ray.step, value_color(sample_scalar(hit)).rgb);
        }
        let s = dvr_sample(d);
        if (s.a > 0.0) {
            let a = corrected_alpha(s.a, ray.step_world);
            accum += (1.0 - alpha_acc) * a * s.rgb;
            alpha_acc += a * (1.0 - alpha_acc);
            if (alpha_acc >= 0.99) {
                break;
            }
        }
        prev = pos;
        pos += ray.step;
    }
    return vec4<f32>(accum, alpha_acc);
}

// DVR of RGB composite colors: opacity follows brightness.
fn dvr_composite(ray: Ray) -> vec4<f32> {
    var pos = ray.start;
    var accum = vec3<f32>(0.0);
    var alpha_acc: f32 = 0.0;
    let alpha_exponent = 1.0 / max(uniforms.absorption, 0.1);

    for (var i = 0; i < ray.count; i = i + 1) {
        let s = sample_composite(pos);
        if (s.a > 0.001) {
            if (uniforms.transparency == 0u) {
                return shade_surface(pos, ray.step, s.rgb);
            }
            let a = corrected_alpha(clamp(pow(s.a, alpha_exponent), 0.01, 1.0), ray.step_world);
            accum += (1.0 - alpha_acc) * a * s.rgb;
            alpha_acc += a * (1.0 - alpha_acc);
            if (alpha_acc >= 0.99) {
                break;
            }
        }
        pos += ray.step;
    }
    return vec4<f32>(accum, alpha_acc);
}

// 4. Categorical / Label Segmented Surface (Binary Mask Normals)
fn label_iso(ray: Ray) -> vec4<f32> {
    var pos = ray.start;
    var prev_pos = ray.start;
    var hit_label: f32 = -1.0;
    var hit_pos = ray.start;

    for (var i = 0; i < ray.count; i = i + 1) {
        let label = sample_exact(pos);
        if (!is_missing(label) && label >= 0.5) {
            hit_label = label;
            // Bisection on the foreground crossing
            var lo = prev_pos;
            var hi = pos;
            for (var step = 0; step < 3; step = step + 1) {
                let mid = 0.5 * (lo + hi);
                if (sample_foreground(mid) >= 0.5) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            hit_pos = hi;
            break;
        }
        prev_pos = pos;
        pos += ray.step;
    }

    if (hit_label < 0.5) {
        return vec4<f32>(0.0);
    }
    let N = sobel_normal_mask(hit_pos);
    let shaded = blinnphong(N, normalize(-ray.step), uniforms.light_direction, value_color(hit_label).rgb);
    return vec4<f32>(shaded, 1.0);
}
