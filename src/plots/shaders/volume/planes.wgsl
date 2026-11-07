// The slicer's planes mode: orthoslices. Each shown plane (`uniforms.slice`,
// unit-box positions in `lo.xyz`, negative when hidden) is drawn where the
// ray crosses it inside the box, colored as the voxel there (exact, so
// voxels stay crisp), always opaque (the layer's opacity and alpha curve
// left out) and shaded a little by how squarely it is seen; the nearest
// plane with a color wins, so missing data shows the planes behind.

// Color of the voxel at `p`, alpha 0 where nothing is drawn: missing voxels
// without the NaN color, values outside the color range without their clip
// color, black composites (as the hover's `VolumeSampler::is_visible`).
fn plane_color(p: vec3<f32>) -> vec4<f32> {
    if (is_composite()) {
        let t = exact_texel(p);
        let c = textureLoad(volume_values, t, 0).rgb;
        if (texel_missing(t) || all(c == vec3<f32>(0.0))) {
            return vec4<f32>(0.0);
        }
        return vec4<f32>(c, 1.0);
    }
    let d = sample_exact(p);
    let c = uniforms.color;
    let hidden = select(
        (d < c.cmin && c.use_lowclip == 0u) || (d > c.cmax && c.use_highclip == 0u),
        c.use_nan_color == 0u,
        is_missing(d),
    );
    if (hidden) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(evaluate_plot_color(d, c).rgb, 1.0);
}

// The nearest drawn plane hit between `t_in` and `t_out` along `eye + t dir`,
// alpha 0 when none.
fn orthoslices(eye: vec3<f32>, dir: vec3<f32>, t_in: f32, t_out: f32) -> vec4<f32> {
    let planes = uniforms.slice.lo.xyz;
    let unit_dir = normalize(dir * world_scale());
    var best = t_out;
    var color = vec4<f32>(0.0);
    for (var k = 0u; k < 3u; k = k + 1u) {
        if (planes[k] < 0.0 || abs(dir[k]) < 1e-6) {
            continue;
        }
        let t = (planes[k] - eye[k]) / dir[k];
        if (t < t_in || t > best) {
            continue;
        }
        let c = plane_color(eye + t * dir);
        if (c.a > 0.0) {
            best = t;
            color = vec4<f32>(c.rgb * (0.75 + 0.25 * abs(unit_dir[k])), 1.0);
        }
    }
    return color;
}
