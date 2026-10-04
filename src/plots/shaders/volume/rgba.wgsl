// Optical RGBA models: absorption (5), additive emission (6) and indexed palettes (7).

// Straight RGBA of a sample: composite color with brightness as alpha, or the
// display color of the scalar.
fn sample_rgba(p: vec3<f32>) -> vec4<f32> {
    if (is_composite()) {
        return sample_composite(p);
    }
    return value_color(sample_scalar(p));
}

// Bricks that can show anything in the RGBA modes: lit composite voxels
// (above `level` brightness), else any data or a shown NaN color.
fn rgba_skip_kind() -> u32 {
    return select(SKIP_VALID, SKIP_ABOVE, is_composite());
}

// 5. Optical Absorption RGBA
fn absorptionrgba(ray: Ray) -> vec4<f32> {
    var transmittance: f32 = 1.0;
    var color_sum = vec3<f32>(0.0);
    let kind = rgba_skip_kind();
    let level = select(0.0, 0.05, uniforms.transparency == 0u);

    for (var i = skip_empty(ray, 0, kind, level); i < ray.count; i = skip_empty(ray, i + 1, kind, level)) {
        let pos = sample_pos(ray, i);
        let s = sample_rgba(pos);
        if (uniforms.transparency == 0u) {
            if (s.a > 0.05) {
                return opaque_surface(ray, previous_pos(ray, i), pos, SURFACE_OPAQUE, i == 0);
            }
        } else {
            let opacity = extinction_alpha(s.a, ray.step_world);
            color_sum += (transmittance * opacity) * s.rgb;
            transmittance *= 1.0 - opacity;
            if (transmittance <= 0.01) {
                break;
            }
        }
    }
    if (uniforms.transparency == 0u) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(color_sum, 1.0 - transmittance);
}

// 6. Additive RGBA (Volume Emission)
fn additivergba(ray: Ray) -> vec4<f32> {
    var integrated = vec4<f32>(0.0);
    let kind = rgba_skip_kind();

    for (var i = skip_empty(ray, 0, kind, 0.0); i < ray.count; i = skip_empty(ray, i + 1, kind, 0.0)) {
        let density = uniforms.absorption * ray.step_world * sample_rgba(sample_pos(ray, i));
        integrated = 1.0 - (1.0 - integrated) * (1.0 - clamp(density, vec4<f32>(0.0), vec4<f32>(1.0)));
        if (min(min(integrated.r, integrated.g), min(integrated.b, integrated.a)) >= 0.99) {
            break;
        }
    }
    return premultiply(integrated);
}

// 7. Volume Indexed RGBA (palette-indexed materials; exact voxels, missing skipped)
fn volumeindexedrgba(ray: Ray) -> vec4<f32> {
    var transmittance: f32 = 1.0;
    var color_sum = vec3<f32>(0.0);

    for (var i = skip_empty(ray, 0, SKIP_VALID, 0.0); i < ray.count; i = skip_empty(ray, i + 1, SKIP_VALID, 0.0)) {
        let raw = sample_exact(sample_pos(ray, i));
        if (!is_missing(raw)) {
            let index = max(i32(raw) - 1, 0);
            let rgb = transfer_at(clamp(f32(index) / 255.0, 0.0, 1.0)).rgb;
            let opacity = extinction_alpha(1.0, ray.step_world);
            color_sum += (transmittance * opacity) * rgb;
            transmittance *= 1.0 - opacity;
            if (transmittance <= 0.01) {
                break;
            }
        }
    }
    return vec4<f32>(color_sum, 1.0 - transmittance);
}
