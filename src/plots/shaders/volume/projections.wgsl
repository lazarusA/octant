// Projection modes: maximum (1), minimum (2) and average (3) intensity, on
// nearest voxels as the original shader drew them (straight alpha).

// Opaque or translucent output of a projected color, per the transparency toggle.
fn projection_output(col: vec4<f32>) -> vec4<f32> {
    if (uniforms.transparency == 0u) {
        return vec4<f32>(col.rgb, 1.0);
    }
    return col;
}

// 1. Maximum Intensity Projection (MIP) of RGB composite colors, per channel,
// over every voxel the ray crosses.
fn mip_composite(ray: Ray) -> vec4<f32> {
    var max_rgb = vec3<f32>(0.0);
    var any_hit = false;
    var w = walk_begin(ray);
    for (var i = 0; i < walk_limit(w); i = i + 1) {
        if (!walk_missing(w)) {
            let c = walk_texel(w);
            max_rgb = max(max_rgb, c.rgb);
            any_hit = any_hit || c.a > 0.001;
        }
        if (!walk_step(&w)) {
            break;
        }
    }
    if (!any_hit) {
        return vec4<f32>(0.0);
    }
    let alpha = select(1.0, max(max_rgb.r, max(max_rgb.g, max_rgb.b)), uniforms.transparency == 1u);
    return vec4<f32>(max_rgb, alpha);
}

// 1. Maximum Intensity Projection (MIP) with density-weighted attenuation,
// over every voxel the ray crosses (the walk), so no maximum is stepped over.
fn mip(ray: Ray) -> vec4<f32> {
    if (is_composite()) {
        return mip_composite(ray);
    }
    var maximum: f32 = -1e30;
    var max_raw: f32 = -1e30;
    var density_sum: f32 = 0.0;
    let cmax = uniforms.color.cmax;
    let highclip_visible = uniforms.color.highclip_color.a > 0.0;
    let range = max(cmax - uniforms.color.cmin, 0.0001);
    let attenuate = uniforms.attenuation > 0.0;

    var w = walk_begin(ray);
    var last = vec3<i32>(-1);
    var open = true;
    for (var i = 0; i < walk_limit(w); i = i + 1) {
        // Blocks no higher than the maximum change nothing (attenuation
        // needs every voxel's density).
        if (!attenuate && !walk_block_open(w, &last, &open, OPEN_ABOVE, maximum)) {
            if (!walk_skip_block(&w)) {
                break;
            }
            continue;
        }
        if (!walk_missing(w)) {
            let density = walk_texel(w).r;
            var attenuated = density;
            if (attenuate) {
                // The mean normalized density so far: each voxel weighs the
                // share of the ray inside it, as equal samples did.
                let norm_density = clamp((density - uniforms.color.cmin) / range, 0.0, 1.0);
                density_sum += norm_density * (min(walk_exit(w), 1.0) - w.t);
                attenuated = density * exp(-uniforms.attenuation * density_sum);
            }
            let consider_sample = (density <= cmax) || highclip_visible;
            if (consider_sample && (attenuated > maximum)) {
                maximum = attenuated;
                max_raw = density;
            }
            // Past cmax every larger value shows the same color.
            if (!attenuate && max_raw > cmax) {
                break;
            }
        }
        if (!walk_step(&w)) {
            break;
        }
    }
    if (max_raw == -1e30) {
        return vec4<f32>(0.0);
    }
    return projection_output(evaluate_plot_color(max_raw, uniforms.color));
}

// 2. Minimum Intensity Projection (MinIP). Samples each brick on its own
// step and, unless values below the range can be the minimum shown (a
// visible low clip color), skips blocks wholly below cmin
// (`next_open_sample`; values above cmax still draw, in the top color).
fn minip(ray: Ray) -> vec4<f32> {
    var minimum: f32 = 1e30;
    var any_hit = false;
    let lowclip_visible = uniforms.color.lowclip_color.a > 0.0;
    var blocks = sample_blocks(ray);

    for (var i = 0; i < ray.count; i = i + 1) {
        var next = i;
        if (lowclip_visible) {
            next = next_strided_sample(&blocks, i);
        } else {
            next = next_open_sample(&blocks, i, OPEN_AT_LEAST);
        }
        if (next != i) {
            i = next - 1;
            continue;
        }
        let density = blocks_exact(&blocks, sample_pos(ray, i));
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

// 3. Average / Mean Intensity Projection (Radiographic Column Transmission).
// Skips blocks drawing nothing (no value in the range counts) and samples
// each brick on its own step, a sample weighing the steps it stands for.
fn average_projection(ray: Ray) -> vec4<f32> {
    var sum_val: f32 = 0.0;
    var valid_count: f32 = 0.0;
    var blocks = sample_blocks(ray);

    for (var i = 0; i < ray.count; i = i + 1) {
        let next = next_open_sample(&blocks, i, OPEN_DRAWN);
        if (next != i) {
            i = next - 1;
            continue;
        }
        let density = blocks_exact(&blocks, sample_pos(ray, i));
        if (!is_missing(density) && density >= uniforms.color.cmin && density <= uniforms.color.cmax) {
            let span = sample_span(&blocks);
            sum_val += density * span;
            valid_count += span;
        }
    }
    if (valid_count <= 0.0) {
        return vec4<f32>(0.0);
    }
    return projection_output(evaluate_plot_color(sum_val / valid_count, uniforms.color));
}
