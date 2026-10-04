// Shared helpers of the classic modes (1-7): nearest-voxel reads, normals and
// Blinn-Phong lighting as the original single-file shader had them.

// Display color of the voxel holding `p`: its composite color (alpha = the
// brightest channel), or the colormapped scalar; the NaN color where missing.
fn exact_rgba(p: vec3<f32>) -> vec4<f32> {
    let t = exact_texel(p);
    if (texel_missing(t)) {
        return select(vec4<f32>(0.0), uniforms.color.nan_color, uniforms.color.use_nan_color == 1u);
    }
    let v = textureLoad(volume_values, t, 0);
    if (is_composite()) {
        return v;
    }
    return evaluate_plot_color(v.r, uniforms.color);
}

// Intensity of the voxel holding `p` (composite brightness), zero where missing.
fn exact_intensity(p: vec3<f32>) -> f32 {
    let t = exact_texel(p);
    if (texel_missing(t)) {
        return 0.0;
    }
    let v = textureLoad(volume_values, t, 0);
    return select(v.r, v.a, is_composite());
}

// 1 where the voxel holding `p` is label foreground, else 0.
fn exact_foreground(p: vec3<f32>) -> f32 {
    let t = exact_texel(p);
    if (texel_missing(t)) {
        return 0.0;
    }
    let v = textureLoad(volume_values, t, 0);
    let inside = select(v.r >= 0.5, v.a >= 0.01, is_composite());
    return select(0.0, 1.0, inside);
}

// 26-neighbor 3D Sobel-Feldman normal for binary segmentation foreground masks
fn sobel_normal_mask(uvw: vec3<f32>) -> vec3<f32> {
    let step = voxel_step();
    var G = vec3<f32>(0.0);
    for (var i = -1; i <= 1; i = i + 1) {
        for (var j = -1; j <= 1; j = j + 1) {
            for (var k = -1; k <= 1; k = k + 1) {
                if (i == 0 && j == 0 && k == 0) { continue; }
                let tap = clamp(uvw + vec3<f32>(f32(i), f32(j), f32(k)) * step, vec3<f32>(0.0), vec3<f32>(1.0));
                let val = exact_foreground(tap);
                let on_axis_x = f32(j == 0 && k == 0);
                let face_x    = f32(j == 0 || k == 0);
                let wx = f32(-i) * (1.0 + face_x + 2.0 * on_axis_x);
                let on_axis_y = f32(i == 0 && k == 0);
                let face_y    = f32(i == 0 || k == 0);
                let wy = f32(-j) * (1.0 + face_y + 2.0 * on_axis_y);
                let on_axis_z = f32(i == 0 && j == 0);
                let face_z    = f32(i == 0 || j == 0);
                let wz = f32(-k) * (1.0 + face_z + 2.0 * on_axis_z);
                G += val * vec3<f32>(wx, wy, wz);
            }
        }
    }
    let len = length(G);
    if (len < 0.00001) {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return normalize(G);
}

// Fast 6-tap central finite-difference gradient for normal estimation
fn central_normal(uvw: vec3<f32>) -> vec3<f32> {
    let step = voxel_step();
    let lo = vec3<f32>(0.0);
    let hi = vec3<f32>(1.0);
    let x1 = exact_intensity(clamp(uvw + vec3<f32>(step.x, 0.0, 0.0), lo, hi));
    let x0 = exact_intensity(clamp(uvw - vec3<f32>(step.x, 0.0, 0.0), lo, hi));
    let y1 = exact_intensity(clamp(uvw + vec3<f32>(0.0, step.y, 0.0), lo, hi));
    let y0 = exact_intensity(clamp(uvw - vec3<f32>(0.0, step.y, 0.0), lo, hi));
    let z1 = exact_intensity(clamp(uvw + vec3<f32>(0.0, 0.0, step.z), lo, hi));
    let z0 = exact_intensity(clamp(uvw - vec3<f32>(0.0, 0.0, step.z), lo, hi));

    let G = vec3<f32>(-(x1 - x0), -(y1 - y0), -(z1 - z0));
    let len = length(G);
    if (len < 0.00001) {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return G / len;
}

fn classic_blinnphong(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, color: vec3<f32>) -> vec3<f32> {
    let light_dir = select(normalize(L), normalize(vec3<f32>(0.4, 0.8, 0.6)), length(L) < 0.001);
    let diff_coeff = max(dot(light_dir, N), 0.0) + max(dot(light_dir, -N), 0.0) * 0.4;
    let H = normalize(light_dir + V);
    let spec_coeff = pow(max(dot(H, N), 0.0), uniforms.shininess);

    let ambient = max(uniforms.ambient, vec3<f32>(0.35));
    return ambient * color + uniforms.diffuse * diff_coeff * color + uniforms.light_color * uniforms.specular * spec_coeff;
}

// Unit-box direction toward the viewer, as the classic modes light with it.
fn camera_dir(ray: Ray) -> vec3<f32> {
    return normalize(-ray.step);
}
