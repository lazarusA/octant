// Classic surface and optical modes on nearest voxels (straight alpha):
// label surface (4, on the voxel walk), absorption (5), additive emission (6), indexed palettes (7).

// Straight color of a label voxel, alpha 0 where nothing is drawn: values
// in the color range, outside it only with their clip color on, missing
// voxels only with the NaN color on; composites wherever they are lit.
fn label_color(missing: bool, v: vec4<f32>) -> vec4<f32> {
    let c = uniforms.color;
    if (missing) {
        return select(vec4<f32>(0.0), c.nan_color, c.use_nan_color == 1u);
    }
    if (is_composite()) {
        return select(vec4<f32>(0.0), vec4<f32>(v.rgb, 1.0), v.a > 0.001);
    }
    let d = v.r;
    if ((d < c.cmin && c.use_lowclip == 0u) || (d > c.cmax && c.use_highclip == 0u)) {
        return vec4<f32>(0.0);
    }
    return evaluate_plot_color(d, c);
}

// 4. Categorical label surface: every voxel the ray crosses (the walk), a
// surface wherever the drawn value changes, class boundaries included, lit
// by the face the ray crosses into it (the entry face at the volume's edge
// or a cut). Opaque, or with Transparency on each surface at the layer's
// opacity, so the boundaries inside show through.
fn label_iso(ray: Ray) -> vec4<f32> {
    let opaque = uniforms.transparency == 0u;
    var accum = vec3<f32>(0.0);
    var alpha: f32 = 0.0;
    var prev = vec4<f32>(0.0);
    var prev_drawn = false;
    var w = walk_begin(ray);
    for (var i = 0; i < walk_limit(w); i = i + 1) {
        let missing = walk_missing(w);
        let texel = select(walk_texel(w), vec4<f32>(MISSING), missing);
        let c = label_color(missing, texel);
        let drawn = c.a > 0.0;
        if (drawn && (!prev_drawn || any(texel != prev))) {
            let n = walk_face_normal(w, ray.entry_normal);
            let rgb = shade_surface(ray, n, c.rgb).rgb;
            let a = select(c.a, 1.0, opaque);
            accum += (1.0 - alpha) * a * rgb;
            alpha += a * (1.0 - alpha);
            if (alpha >= 0.99) {
                break;
            }
        }
        prev = texel;
        prev_drawn = drawn;
        if (!walk_step(&w)) {
            break;
        }
    }
    if (alpha <= 0.0) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(accum / alpha, alpha);
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
