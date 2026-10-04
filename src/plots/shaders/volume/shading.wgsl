// Surface shading: gradient normals and Blinn-Phong lighting.

// Scalar driving surface normals: the trilinear value, or composite brightness.
fn sample_intensity(p: vec3<f32>) -> f32 {
    if (is_composite()) {
        return sample_composite(p).a;
    }
    let s = sample_scalar(p);
    return select(s, 0.0, is_missing(s));
}

// 1 inside the foreground of a label mask (exact voxels), else 0.
fn sample_foreground(p: vec3<f32>) -> f32 {
    if (is_composite()) {
        return select(0.0, 1.0, sample_composite(p).a >= 0.01);
    }
    let label = sample_exact(p);
    return select(0.0, 1.0, !is_missing(label) && label >= 0.5);
}

fn voxel_step() -> vec3<f32> {
    return 1.0 / volume_dims();
}

// 26-neighbor 3D Sobel-Feldman normal for binary segmentation foreground masks
fn sobel_normal_mask(uvw: vec3<f32>) -> vec3<f32> {
    let step = voxel_step();
    var G = vec3<f32>(0.0);
    for (var i = -1; i <= 1; i = i + 1) {
        for (var j = -1; j <= 1; j = j + 1) {
            for (var k = -1; k <= 1; k = k + 1) {
                if (i == 0 && j == 0 && k == 0) { continue; }
                let sample_pos = clamp(uvw + vec3<f32>(f32(i), f32(j), f32(k)) * step, vec3<f32>(0.0), vec3<f32>(1.0));
                let val = sample_foreground(sample_pos);
                let wx = f32(-i) * (1.0 + f32(j == 0 || k == 0) + 2.0 * f32(j == 0 && k == 0));
                let wy = f32(-j) * (1.0 + f32(i == 0 || k == 0) + 2.0 * f32(i == 0 && k == 0));
                let wz = f32(-k) * (1.0 + f32(i == 0 || j == 0) + 2.0 * f32(i == 0 && j == 0));
                G += val * vec3<f32>(wx, wy, wz);
            }
        }
    }
    let len = length(G);
    if (len < 0.00001) {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return G / len;
}

// 6-tap central-difference normal on the trilinear field
fn central_normal(uvw: vec3<f32>) -> vec3<f32> {
    let step = voxel_step();
    let dx = vec3<f32>(step.x, 0.0, 0.0);
    let dy = vec3<f32>(0.0, step.y, 0.0);
    let dz = vec3<f32>(0.0, 0.0, step.z);
    let G = -vec3<f32>(
        sample_intensity(uvw + dx) - sample_intensity(uvw - dx),
        sample_intensity(uvw + dy) - sample_intensity(uvw - dy),
        sample_intensity(uvw + dz) - sample_intensity(uvw - dz),
    );
    let len = length(G);
    if (len < 0.00001) {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return G / len;
}

fn blinnphong(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, color: vec3<f32>) -> vec3<f32> {
    let light_dir = select(normalize(L), normalize(vec3<f32>(0.4, 0.8, 0.6)), length(L) < 0.001);
    let diff_coeff = max(dot(light_dir, N), 0.0) + max(dot(light_dir, -N), 0.0) * 0.4;
    let H = normalize(light_dir + V);
    let spec_coeff = pow(max(dot(H, N), 0.0), uniforms.shininess);

    let ambient = max(uniforms.ambient, vec3<f32>(0.35));
    return ambient * color + uniforms.diffuse * diff_coeff * color + uniforms.light_color * uniforms.specular * spec_coeff;
}

// Lit surface color at `p`, seen along `ray_dir`.
fn shade_surface(p: vec3<f32>, ray_dir: vec3<f32>, color: vec3<f32>) -> vec4<f32> {
    let lit = blinnphong(central_normal(p), normalize(-ray_dir), uniforms.light_direction, color);
    return vec4<f32>(lit, 1.0);
}
