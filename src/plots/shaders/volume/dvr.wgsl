// Direct volume rendering (mode 0): front-to-back compositing, or the first
// opaque surface with transparency off.

// What makes a sample part of an opaque surface.
const SURFACE_RANGE: u32 = 0u;  // scalar inside [cmin, cmax]
const SURFACE_BRIGHT: u32 = 1u; // composite brightness above 0.001

fn in_display_range(d: f32) -> bool {
    return !is_missing(d) && d >= uniforms.color.cmin && d <= uniforms.color.cmax;
}

fn is_inside(p: vec3<f32>, kind: u32) -> bool {
    if (kind == SURFACE_RANGE) {
        return in_display_range(sample_scalar(p));
    }
    return sample_composite(p).a > 0.001;
}

fn surface_color(p: vec3<f32>, kind: u32) -> vec3<f32> {
    if (kind == SURFACE_RANGE) {
        return evaluate_plot_color(sample_scalar(p), uniforms.color).rgb;
    }
    return sample_composite(p).rgb;
}

// Bisects between `outside` and `inside` to place the surface between steps.
fn refine_hit(outside: vec3<f32>, inside: vec3<f32>, kind: u32) -> vec3<f32> {
    var lo = outside;
    var hi = inside;
    for (var i = 0; i < 6; i = i + 1) {
        let mid = 0.5 * (lo + hi);
        if (is_inside(mid, kind)) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    return hi;
}

// Lit first-hit surface between the previous sample `prev` and `pos` (step
// `first` starts from the ray entry): where the volume is cut open at the
// entry face the face itself is the surface, else the refined crossing with
// the field's gradient normal.
fn opaque_surface(ray: Ray, prev: vec3<f32>, pos: vec3<f32>, kind: u32, first: bool) -> vec4<f32> {
    if (first && is_inside(ray.entry, kind)) {
        return shade_surface(ray, ray.entry_normal, surface_color(ray.entry, kind));
    }
    let hit = refine_hit(prev, pos, kind);
    let n = facing_normal(gradient_intensity(hit), ray.view);
    return shade_surface(ray, n, surface_color(hit, kind));
}

// DVR opacity at scale position `t` per REFERENCE_STEP: `t^(1/Density)`,
// floored so every in-range sample stays faintly visible.
fn dvr_opacity(t: f32) -> f32 {
    return clamp(pow(clamp(t, 0.001, 1.0), 1.0 / max(uniforms.absorption, 0.1)), 0.01, 1.0);
}

// Straight color of a DVR sample and its opacity over one step: colormap
// color and DVR opacity in range, enabled NaN or clip colors outside it,
// transparent otherwise. Opacities are defined per REFERENCE_STEP.
fn dvr_sample(d: f32, step_world: f32) -> vec4<f32> {
    let c = uniforms.color;
    var user = vec4<f32>(0.0);
    if (is_missing(d)) {
        user = select(user, c.nan_color, c.use_nan_color == 1u);
    } else if (d < c.cmin) {
        user = select(user, c.lowclip_color, c.use_lowclip == 1u);
    } else if (d > c.cmax) {
        user = select(user, c.highclip_color, c.use_highclip == 1u);
    } else {
        let t = evaluate_scaled_norm(d, c.cmin, c.cmax, c.scale_type, c.scale_param);
        let e = evaluate_plot_color(d, c);
        user = vec4<f32>(e.rgb, dvr_opacity(t) * e.a);
    }
    return vec4<f32>(user.rgb, corrected_alpha(user.a, step_world));
}

// Samples this faint skip the lighting gradient.
const LIT_ALPHA_MIN: f32 = 0.002;

// 0. Direct Volume Rendering (DVR) with front-to-back alpha compositing
fn volume_dvr(ray: Ray) -> vec4<f32> {
    if (is_composite()) {
        return dvr_composite(ray);
    }
    let range = uniforms.color.cmax - uniforms.color.cmin;
    var accum = vec3<f32>(0.0);
    var alpha_acc: f32 = 0.0;

    for (var i = 0; i < ray.count; i = i + 1) {
        let pos = sample_pos(ray, i);
        let d = sample_scalar(pos);
        if (uniforms.transparency == 0u && in_display_range(d)) {
            return opaque_surface(ray, previous_pos(ray, i), pos, SURFACE_RANGE, i == 0);
        }
        let s = dvr_sample(d, ray.step_world);
        let a = s.a;
        if (a > 0.0) {
            let lit = uniforms.lighting != 0u && a > LIT_ALPHA_MIN && in_display_range(d);
            let rgb = select(s.rgb, lit_sample(ray, pos, s.rgb, range), lit);
            accum += (1.0 - alpha_acc) * a * rgb;
            alpha_acc += a * (1.0 - alpha_acc);
            if (alpha_acc >= 0.99) {
                break;
            }
        }
    }
    return vec4<f32>(accum, alpha_acc);
}

// DVR of RGB composite colors: opacity follows brightness.
fn dvr_composite(ray: Ray) -> vec4<f32> {
    var accum = vec3<f32>(0.0);
    var alpha_acc: f32 = 0.0;

    for (var i = 0; i < ray.count; i = i + 1) {
        let pos = sample_pos(ray, i);
        let s = sample_composite(pos);
        if (s.a > 0.001) {
            if (uniforms.transparency == 0u) {
                return opaque_surface(ray, previous_pos(ray, i), pos, SURFACE_BRIGHT, i == 0);
            }
            let a = corrected_alpha(dvr_opacity(s.a), ray.step_world);
            let lit = uniforms.lighting != 0u && a > LIT_ALPHA_MIN;
            let rgb = select(s.rgb, lit_sample(ray, pos, s.rgb, 1.0), lit);
            accum += (1.0 - alpha_acc) * a * rgb;
            alpha_acc += a * (1.0 - alpha_acc);
            if (alpha_acc >= 0.99) {
                break;
            }
        }
    }
    return vec4<f32>(accum, alpha_acc);
}
