// Direct volume rendering (mode 0), the categorical label surface (mode 4) and
// the opaque first-hit surfaces they share with absorption (mode 5).

// What makes a sample part of an opaque surface.
const SURFACE_RANGE: u32 = 0u;  // scalar inside [cmin, cmax]
const SURFACE_BRIGHT: u32 = 1u; // composite brightness above 0.001
const SURFACE_OPAQUE: u32 = 2u; // display alpha above 0.05 (absorption)
const SURFACE_MASK: u32 = 3u;   // smooth label foreground

fn in_display_range(d: f32) -> bool {
    return !is_missing(d) && d >= uniforms.color.cmin && d <= uniforms.color.cmax;
}

fn is_inside(p: vec3<f32>, kind: u32) -> bool {
    switch kind {
        case 0u: { return in_display_range(sample_scalar(p)); }
        case 1u: { return sample_composite(p).a > 0.001; }
        case 2u: { return sample_rgba(p).a > 0.05; }
        default: { return mask_smooth(p) >= 0.5; }
    }
}

fn surface_color(p: vec3<f32>, kind: u32) -> vec3<f32> {
    switch kind {
        case 0u: { return value_color(sample_scalar(p)).rgb; }
        case 1u: { return sample_composite(p).rgb; }
        default: { return sample_rgba(p).rgb; }
    }
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

// First-hit surface point between the previous sample `prev` and `pos` (step
// `first` starts from the ray entry), with its normal: where the volume is cut
// open at the entry face the face itself is the surface, else the refined
// crossing with the field's gradient normal. A label march tests exact voxels,
// and the smooth mask can extend about a voxel past them, so the search backs
// up from `prev` one voxel at a time until it is outside.
fn surface_hit(ray: Ray, prev: vec3<f32>, pos: vec3<f32>, kind: u32, first: bool) -> array<vec3<f32>, 2> {
    if (first && is_inside(ray.entry, kind)) {
        return array<vec3<f32>, 2>(ray.entry, ray.entry_normal);
    }
    let voxel_back = ray.step / max(length(ray.step * volume_dims()), 1e-6);
    var outside = prev;
    for (var i = 0; i < 3 && is_inside(outside, kind); i = i + 1) {
        outside -= voxel_back;
    }
    let hit = refine_hit(outside, pos, kind);
    let g = select(gradient_intensity(hit), gradient_mask(hit), kind == SURFACE_MASK);
    return array<vec3<f32>, 2>(hit, facing_normal(g, ray.view));
}

fn opaque_surface(ray: Ray, prev: vec3<f32>, pos: vec3<f32>, kind: u32, first: bool) -> vec4<f32> {
    let hit = surface_hit(ray, prev, pos, kind, first);
    return shade_surface(ray, hit[1], surface_color(hit[0], kind));
}

// Straight color of a DVR sample and its opacity over one step: transfer
// color and extinction in range, enabled NaN or clip colors outside it,
// transparent otherwise.
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
        let tf = transfer_at(scale_position(d));
        return vec4<f32>(tf.rgb, extinction_alpha(tf.a, step_world));
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

    for (var i = skip_empty(ray, 0, SKIP_DVR, 0.0); i < ray.count; i = skip_empty(ray, i + 1, SKIP_DVR, 0.0)) {
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

    for (var i = skip_empty(ray, 0, SKIP_ABOVE, 0.001); i < ray.count; i = skip_empty(ray, i + 1, SKIP_ABOVE, 0.001)) {
        let pos = sample_pos(ray, i);
        let s = sample_composite(pos);
        if (s.a > 0.001) {
            if (uniforms.transparency == 0u) {
                return opaque_surface(ray, previous_pos(ray, i), pos, SURFACE_BRIGHT, i == 0);
            }
            let a = extinction_alpha(s.a * s.a, ray.step_world);
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

// 4. Categorical / Label Segmented Surface: found on exact voxels, placed and
// lit on the smooth filtered mask, colored by the voxel's label.
fn label_iso(ray: Ray) -> vec4<f32> {
    // Bricks matter when they reach the foreground threshold.
    let level = select(0.4999, 0.0099, is_composite());
    for (var i = skip_empty(ray, 0, SKIP_ABOVE, level); i < ray.count; i = skip_empty(ray, i + 1, SKIP_ABOVE, level)) {
        let pos = sample_pos(ray, i);
        let label = sample_exact(pos);
        if (!is_missing(label) && label >= 0.5) {
            let hit = surface_hit(ray, previous_pos(ray, i), pos, SURFACE_MASK, i == 0);
            return shade_surface(ray, hit[1], value_color(label).rgb);
        }
    }
    return vec4<f32>(0.0);
}
