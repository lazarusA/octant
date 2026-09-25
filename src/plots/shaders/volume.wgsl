// =================================================================================================
// Volume Raymarching Shader
//
// Rendering Modes:
//   0: Volume Raymarching (DVR)    - Front-to-back alpha compositing with Beer-Lambert absorption
//   1: Maximum Intensity (MIP)     - Maximum intensity projection with density-weighted attenuation
//   2: Minimum Intensity (MinIP)   - Minimum intensity projection along the ray
//   3: Average Projection (X-ray)  - Average column scalar intensity (radiographic transmission)
//   4: Categorical Label Surface   - Binary foreground mask isosurface for segmented data
//   5: Absorption RGBA             - Classical optical absorption model
//   6: Additive RGBA               - Additive volume emission model
//   7: Indexed Discrete RGBA       - Palette-indexed discrete material rendering
// =================================================================================================

struct Uniforms {
    clip_planes: array<vec4<f32>, 8>,
    light_color: vec3<f32>,
    num_clip_planes: u32,
    ambient: vec3<f32>,
    shininess: f32,
    light_direction: vec3<f32>,
    algorithm: u32,
    isovalue: f32,
    isorange: f32,
    absorption: f32,
    samples: u32,
    diffuse: f32,
    specular: f32,
    attenuation: f32,
    picking: u32,
    object_id: u32,
    rotation_y: f32,
    rotation_x: f32,
    aspect_x: f32,
    aspect_y: f32,
    aspect_z: f32,
    zoom: f32,
    width: u32,
    height: u32,
    depth: u32,
    screen_aspect: f32,
    shift_x: u32,
    shift_y: u32,
    shift_z: u32,
    transparency: u32,
    _pad1: u32,
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> data_buffer: array<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_vert: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Centered object position [-0.5, 0.5] scaled by aspect ratio
    let unit_pos = model.position * 0.5;
    let pos_3d = unit_pos * vec3<f32>(
        uniforms.aspect_x,
        uniforms.aspect_y,
        uniforms.aspect_z
    );

    // 3D Camera rotation around Y and X axes
    let cy = cos(uniforms.rotation_y);
    let sy = sin(uniforms.rotation_y);
    let cx = cos(uniforms.rotation_x);
    let sx = sin(uniforms.rotation_x);

    let pos_y_rot = vec3<f32>(
        cy * pos_3d.x + sy * pos_3d.z,
        pos_3d.y,
        -sy * pos_3d.x + cy * pos_3d.z
    );

    let pos_rot = vec3<f32>(
        pos_y_rot.x,
        cx * pos_y_rot.y - sx * pos_y_rot.z,
        sx * pos_y_rot.y + cx * pos_y_rot.z
    );

    out.frag_vert = pos_3d;

    let cam_dist = clamp(uniforms.zoom, 0.1, 10.0);
    let cam_z = pos_rot.z - cam_dist;
    let dist_positive = max(-cam_z, 0.001);

    let fov_scale = 1.6;
    let screen_asp = max(uniforms.screen_aspect, 0.1);
    let proj_x = (pos_rot.x * fov_scale) / screen_asp;
    let proj_y = pos_rot.y * fov_scale;

    let z_near = 0.01;
    let z_far = 50.0;
    let proj_z = (z_far / (z_far - z_near)) * dist_positive - (z_far * z_near / (z_far - z_near));

    out.position = vec4<f32>(proj_x, proj_y, proj_z, dist_positive);
    return out;
}

fn unpack_rgba_f32(s: f32) -> vec4<f32> {
    let packed = u32(s);
    let r = f32(packed & 0xFFu) / 255.0;
    let g = f32((packed >> 8u) & 0xFFu) / 255.0;
    let b = f32((packed >> 16u) & 0xFFu) / 255.0;
    let a = max(r, max(g, b));
    return vec4<f32>(r, g, b, a);
}

fn color_lookup(intensity: f32) -> vec4<f32> {
    return evaluate_plot_color(intensity, uniforms.color);
}

fn color_lookup_indexed(colormap: u32, index: i32) -> vec4<f32> {
    let norm = clamp(f32(max(index, 0)) / 255.0, 0.0, 1.0);
    let rgb = sample_colormap(colormap, norm);
    return vec4<f32>(rgb, 1.0);
}

fn sample_volume_scalar(texCoord: vec3<f32>) -> f32 {
    if (texCoord.x < 0.0 || texCoord.x > 1.0 || texCoord.y < 0.0 || texCoord.y > 1.0 || texCoord.z < 0.0 || texCoord.z > 1.0) {
        return 0.0;
    }
    let grid_w = max(uniforms.width, 1u);
    let grid_h = max(uniforms.height, 1u);
    let grid_d = max(uniforms.depth, 1u);
    let total_len = arrayLength(&data_buffer);

    let norm_y = 1.0 - texCoord.y;
    let norm_z = 1.0 - texCoord.z;
    let base_x = u32(texCoord.x * f32(grid_w - 1u));
    let base_y = u32(norm_y * f32(grid_h - 1u));
    let base_z = u32(norm_z * f32(grid_d - 1u));

    let gx = (base_x + uniforms.shift_x) % grid_w;
    let gy = (base_y + uniforms.shift_y) % grid_h;
    let gz = (base_z + uniforms.shift_z) % grid_d;

    let idx = min(gz * grid_h * grid_w + gy * grid_w + gx, total_len - 1u);
    return data_buffer[idx];
}

fn sample_volume_rgba(pos: vec3<f32>) -> vec4<f32> {
    let s = sample_volume_scalar(pos);
    if (uniforms.color.colormap == 1000u) {
        let is_nan_val = (s != s || abs(s) > 1e30);
        if (is_nan_val) {
            return select(vec4<f32>(0.0), uniforms.color.nan_color, uniforms.color.use_nan_color == 1u);
        }
        return unpack_rgba_f32(s);
    }
    return evaluate_plot_color(s, uniforms.color);
}

fn sample_foreground(pos: vec3<f32>) -> f32 {
    let raw = sample_volume_scalar(pos);
    let is_nan_val = (raw != raw || abs(raw) > 1e30);
    if (is_nan_val || raw < 0.5) {
        return 0.0;
    }
    return 1.0;
}


// 26-neighbor 3D Sobel-Feldman normal for binary segmentation foreground masks
fn sobel_normal_mask(uvw: vec3<f32>) -> vec3<f32> {
    let grid_w = f32(max(uniforms.width, 1u));
    let grid_h = f32(max(uniforms.height, 1u));
    let grid_d = f32(max(uniforms.depth, 1u));
    let step = vec3<f32>(1.0 / grid_w, 1.0 / grid_h, 1.0 / grid_d);

    var G = vec3<f32>(0.0);
    for (var i = -1; i <= 1; i = i + 1) {
        for (var j = -1; j <= 1; j = j + 1) {
            for (var k = -1; k <= 1; k = k + 1) {
                if (i == 0 && j == 0 && k == 0) { continue; }
                let sample_pos = clamp(uvw + vec3<f32>(f32(i), f32(j), f32(k)) * step, vec3<f32>(0.0), vec3<f32>(1.0));
                let val = sample_foreground(sample_pos);
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

// Fast 6-tap central finite-difference gradient for high-performance normal estimation
fn central_normal(uvw: vec3<f32>) -> vec3<f32> {
    let grid_w = f32(max(uniforms.width, 1u));
    let grid_h = f32(max(uniforms.height, 1u));
    let grid_d = f32(max(uniforms.depth, 1u));
    let step = vec3<f32>(1.0 / grid_w, 1.0 / grid_h, 1.0 / grid_d);

    let x1 = sample_volume_scalar(clamp(uvw + vec3<f32>(step.x, 0.0, 0.0), vec3<f32>(0.0), vec3<f32>(1.0)));
    let x0 = sample_volume_scalar(clamp(uvw - vec3<f32>(step.x, 0.0, 0.0), vec3<f32>(0.0), vec3<f32>(1.0)));
    let y1 = sample_volume_scalar(clamp(uvw + vec3<f32>(0.0, step.y, 0.0), vec3<f32>(0.0), vec3<f32>(1.0)));
    let y0 = sample_volume_scalar(clamp(uvw - vec3<f32>(0.0, step.y, 0.0), vec3<f32>(0.0), vec3<f32>(1.0)));
    let z1 = sample_volume_scalar(clamp(uvw + vec3<f32>(0.0, 0.0, step.z), vec3<f32>(0.0), vec3<f32>(1.0)));
    let z0 = sample_volume_scalar(clamp(uvw - vec3<f32>(0.0, 0.0, step.z), vec3<f32>(0.0), vec3<f32>(1.0)));

    let G = vec3<f32>(-(x1 - x0), -(y1 - y0), -(z1 - z0));
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

// 0. Default Mode: Direct Volume Rendering (DVR) with Alpha Compositing
fn volume_dvr(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front;
    let samples_count = i32(max(uniforms.samples, 8u));
    var accumColor = vec3<f32>(0.0);
    var alphaAcc: f32 = 0.0;
    let threshold_min = uniforms.color.cmin;
    let threshold_max = uniforms.color.cmax;
    let camdir = normalize(-dir);

    for (var i = 0; i < samples_count; i = i + 1) {
        let d = sample_volume_scalar(pos);
        let is_nan_sample = (d != d || abs(d) > 1e30);
        let is_low_sample = (d < threshold_min) && !is_nan_sample;
        let is_high_sample = (d > threshold_max) && !is_nan_sample;
        let is_in_bounds = (d >= threshold_min && d <= threshold_max) && !is_nan_sample;

        var col = vec3<f32>(0.0);
        var alpha: f32 = 0.0;

        if (is_nan_sample && uniforms.color.use_nan_color == 1u) {
            col = uniforms.color.nan_color.rgb;
            alpha = uniforms.color.nan_color.a;
        } else if (is_low_sample && uniforms.color.use_lowclip == 1u) {
            col = uniforms.color.lowclip_color.rgb;
            alpha = uniforms.color.lowclip_color.a;
        } else if (is_high_sample && uniforms.color.use_highclip == 1u) {
            col = uniforms.color.highclip_color.rgb;
            alpha = uniforms.color.highclip_color.a;
        } else if (is_in_bounds) {
            if (uniforms.color.colormap == 1000u) {
                let unpacked = unpack_rgba_f32(d);
                col = unpacked.rgb;
                let intensity = unpacked.a;
                if (intensity > 0.001) {
                    if (uniforms.transparency == 1u) {
                        let alpha_exponent = max(uniforms.absorption, 0.1);
                        alpha = clamp(pow(intensity, 1.0 / alpha_exponent), 0.01, 1.0);
                    } else {
                        let N = central_normal(pos);
                        let shaded = blinnphong(N, camdir, uniforms.light_direction, col);
                        return vec4<f32>(shaded, 1.0);
                    }
                }
            } else {
                let sample_color = evaluate_plot_color(d, uniforms.color);
                col = sample_color.rgb;

                if (uniforms.transparency == 1u) {
                    let range = max(threshold_max - threshold_min, 0.0001);
                    let sampLoc = clamp((d - threshold_min) / range, 0.001, 1.0);
                    let alpha_exponent = max(uniforms.absorption, 0.1);
                    alpha = clamp(pow(sampLoc, 1.0 / alpha_exponent), 0.01, 1.0);
                } else {
                    let N = central_normal(pos);
                    let shaded = blinnphong(N, camdir, uniforms.light_direction, col);
                    return vec4<f32>(shaded, 1.0);
                }
            }
        }

        if (alpha > 0.0) {
            accumColor = accumColor + (1.0 - alphaAcc) * alpha * col;
            alphaAcc = alphaAcc + alpha * (1.0 - alphaAcc);

            if (alphaAcc >= 0.99) {
                break;
            }
        }
        pos = pos + dir;
    }

    return vec4<f32>(accumColor, alphaAcc);
}

// 1. Maximum Intensity Projection (MIP) with density-weighted attenuation
fn mip(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front + dir;
    let samples_count = i32(max(uniforms.samples, 8u));

    if (uniforms.color.colormap == 1000u) {
        var max_rgb = vec3<f32>(0.0);
        var any_hit = false;
        for (var i = 0; i < samples_count; i = i + 1) {
            let density = sample_volume_scalar(pos);
            let is_nan_val = (density != density || abs(density) > 1e30);
            if (!is_nan_val) {
                let unpacked = unpack_rgba_f32(density);
                max_rgb = max(max_rgb, unpacked.rgb);
                if (unpacked.a > 0.001) {
                    any_hit = true;
                }
            }
            pos = pos + dir;
        }
        if (!any_hit) {
            return vec4<f32>(0.0);
        }
        let alpha = select(1.0, max(max_rgb.r, max(max_rgb.g, max_rgb.b)), uniforms.transparency == 1u);
        return vec4<f32>(max_rgb, alpha);
    }

    var maximum: f32 = -1e30;
    var max_raw: f32 = -1e30;
    var density_sum: f32 = 0.0;
    let highclip_visible = uniforms.color.highclip_color.a > 0.0;
    let range = max(uniforms.color.cmax - uniforms.color.cmin, 0.0001);

    for (var i = 0; i < samples_count; i = i + 1) {
        let density = sample_volume_scalar(pos);
        let is_nan_val = (density != density || abs(density) > 1e30);
        if (!is_nan_val) {
            let norm_density = clamp((density - uniforms.color.cmin) / range, 0.0, 1.0);
            density_sum += norm_density / f32(samples_count);
            let atten = exp(-uniforms.attenuation * density_sum);
            let attenuated_density = density * atten;

            let consider_sample = (density <= uniforms.color.cmax) || highclip_visible;
            if (consider_sample && (attenuated_density > maximum)) {
                maximum = attenuated_density;
                max_raw = density;
            }
        }
        pos = pos + dir;
    }
    if (max_raw == -1e30) {
        return vec4<f32>(0.0);
    }
    let col = color_lookup(max_raw);
    if (uniforms.transparency == 0u) {
        return vec4<f32>(col.rgb, 1.0);
    }
    return col;
}

// 2. Minimum Intensity Projection (MinIP)
fn minip(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front + dir;
    var minimum: f32 = 1e30;
    var any_hit = false;
    let lowclip_visible = uniforms.color.lowclip_color.a > 0.0;
    let samples_count = i32(max(uniforms.samples, 8u));

    for (var i = 0; i < samples_count; i = i + 1) {
        let density = sample_volume_scalar(pos);
        let is_nan_val = (density != density || abs(density) > 1e30);
        if (!is_nan_val) {
            let consider_sample = (density >= uniforms.color.cmin) || lowclip_visible;
            if (consider_sample && (density < minimum)) {
                minimum = density;
                any_hit = true;
            }
        }
        pos = pos + dir;
    }
    if (!any_hit) {
        return vec4<f32>(0.0);
    }
    let col = color_lookup(minimum);
    if (uniforms.transparency == 0u) {
        return vec4<f32>(col.rgb, 1.0);
    }
    return col;
}

// 3. Average / Mean Intensity Projection (Radiographic Column Transmission)
fn average_projection(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front + dir;
    var sum_val: f32 = 0.0;
    var valid_count: f32 = 0.0;
    let samples_count = i32(max(uniforms.samples, 8u));

    for (var i = 0; i < samples_count; i = i + 1) {
        let density = sample_volume_scalar(pos);
        let is_nan_val = (density != density || abs(density) > 1e30);
        if (!is_nan_val && density >= uniforms.color.cmin && density <= uniforms.color.cmax) {
            sum_val += density;
            valid_count += 1.0;
        }
        pos = pos + dir;
    }
    if (valid_count <= 0.0) {
        return vec4<f32>(0.0);
    }
    let mean = sum_val / valid_count;
    let col = color_lookup(mean);
    if (uniforms.transparency == 0u) {
        return vec4<f32>(col.rgb, 1.0);
    }
    return col;
}

// 4. Categorical / Label Segmented Surface (Binary Mask Normals)
fn label_iso(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front;
    let camdir = normalize(-dir);
    let samples_count = i32(max(uniforms.samples, 8u));
    var prev_pos = front;
    var hit_label: f32 = -1.0;
    var hit_pos = front;

    for (var i = 0; i < samples_count; i = i + 1) {
        let label = sample_volume_scalar(pos);
        let is_valid = (label != label || abs(label) > 1e30) == false;
        if (is_valid && label >= 0.5) {
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
        pos = pos + dir;
    }

    if (hit_label < 0.5) {
        return vec4<f32>(0.0);
    }

    let N = sobel_normal_mask(hit_pos);
    let L = uniforms.light_direction;
    let col = evaluate_plot_color(hit_label, uniforms.color);
    let shaded = blinnphong(N, camdir, L, col.rgb);
    return vec4<f32>(shaded, 1.0);
}

// 5. Optical Absorption RGBA
fn absorptionrgba(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front;
    var transmittance: f32 = 1.0;
    var color_sum = vec3<f32>(0.0);
    let step_size = length(dir);
    let samples_count = i32(max(uniforms.samples, 8u));
    let camdir = normalize(-dir);

    for (var i = 0; i < samples_count; i = i + 1) {
        let color_sample = sample_volume_rgba(pos);

        if (uniforms.transparency == 0u) {
            if (color_sample.a > 0.05) {
                let N = central_normal(pos);
                let shaded = blinnphong(N, camdir, uniforms.light_direction, color_sample.rgb);
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

        pos = pos + dir;
    }
    if (uniforms.transparency == 0u) {
        return vec4<f32>(0.0);
    }
    if (1.0 - transmittance <= 0.0) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(color_sum / (1.0 - transmittance), 1.0 - transmittance);
}

// 6. Additive RGBA (Volume Emission)
fn additivergba(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front;
    var integrated_color = vec4<f32>(0.0);
    let step_size = length(dir);
    let samples_count = i32(max(uniforms.samples, 8u));

    for (var i = 0; i < samples_count; i = i + 1) {
        let density = uniforms.absorption * step_size * sample_volume_rgba(pos);
        integrated_color = 1.0 - (1.0 - integrated_color) * (1.0 - density);
        pos = pos + dir;
    }
    return integrated_color;
}

// 7. Volume Indexed RGBA (Palette-indexed materials)
fn volumeindexedrgba(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front;
    var transmittance: f32 = 1.0;
    var color_sum = vec3<f32>(0.0);
    let step_size = length(dir);
    let samples_count = i32(max(uniforms.samples, 8u));

    for (var i = 0; i < samples_count; i = i + 1) {
        let index = i32(sample_volume_scalar(pos)) - 1;
        let color_sample = color_lookup_indexed(uniforms.color.colormap, index);

        let opacity = clamp(step_size * color_sample.a * uniforms.absorption, 0.0, 1.0);
        color_sum = color_sum + (transmittance * opacity) * color_sample.rgb;
        transmittance = transmittance * (1.0 - opacity);

        if (transmittance <= 0.01) {
            break;
        }
        pos = pos + dir;
    }
    if (1.0 - transmittance <= 0.0) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(color_sum / (1.0 - transmittance), 1.0 - transmittance);
}

struct ClipResult {
    clipped: bool,
    p1: vec3<f32>,
    p2: vec3<f32>,
};

fn process_clip_planes(p1_in: vec3<f32>, p2_in: vec3<f32>) -> ClipResult {
    var p1 = p1_in;
    var p2 = p2_in;
    var d1: f32;
    var d2: f32;
    let count = min(uniforms.num_clip_planes, 8u);

    for (var i = 0u; i < count; i = i + 1u) {
        let plane = uniforms.clip_planes[i];
        d1 = dot(p1, plane.xyz) - plane.w;
        d2 = dot(p2, plane.xyz) - plane.w;

        if (d1 < 0.0 && d2 < 0.0) {
            p2 = p1;
            return ClipResult(true, p1, p2);
        }
        else if (d1 < 0.0) {
            p1 = p1 - d1 * (p2 - p1) / (d2 - d1);
        } else if (d2 < 0.0) {
            p2 = p2 - d2 * (p1 - p2) / (d1 - d2);
        }
    }

    return ClipResult(false, p1, p2);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cy = cos(-uniforms.rotation_y);
    let sy = sin(-uniforms.rotation_y);
    let cx = cos(-uniforms.rotation_x);
    let sx = sin(-uniforms.rotation_x);

    let eye_cam = vec3<f32>(0.0, 0.0, uniforms.zoom);
    let eye_x_rot = vec3<f32>(eye_cam.x, cx * eye_cam.y - sx * eye_cam.z, sx * eye_cam.y + cx * eye_cam.z);
    let eye_rot = vec3<f32>(cy * eye_x_rot.x + sy * eye_x_rot.z, eye_x_rot.y, -sy * eye_x_rot.x + cy * eye_x_rot.z);

    let scale_vec = vec3<f32>(
        max(uniforms.aspect_x, 0.001),
        max(uniforms.aspect_y, 0.001),
        max(uniforms.aspect_z, 0.001)
    );

    let eye_unit = vec3<f32>(0.5) + eye_rot / scale_vec;
    let back_position = in.frag_vert / scale_vec + vec3<f32>(0.5);
    let dir = normalize(back_position - eye_unit);

    if (dot(dir, dir) < 0.000001 || dir.x != dir.x || dir.y != dir.y || dir.z != dir.z) {
        discard;
    }

    // Branchless slab intersection with [0, 1]^3 unit bounding box
    let inv_dir = 1.0 / dir;
    let t0 = (vec3<f32>(0.0) - eye_unit) * inv_dir;
    let t1 = (vec3<f32>(1.0) - eye_unit) * inv_dir;
    let tmin = min(t0, t1);
    let tmax = max(t0, t1);

    let t_enter = max(max(tmin.x, tmin.y), tmin.z);
    let t_exit = min(min(tmax.x, tmax.y), tmax.z);

    if (t_enter > t_exit || t_exit < 0.0) {
        discard;
    }

    let start = eye_unit + max(t_enter, 0.0) * dir;
    let stop = eye_unit + t_exit * dir;

    let clip_res = process_clip_planes(start, stop);
    if (clip_res.clipped) {
        discard;
    }

    let step_in_dir = (clip_res.p2 - clip_res.p1) / f32(max(uniforms.samples, 1u));
    let ray_start = clip_res.p1;
    let algo = uniforms.algorithm;

    var color: vec4<f32>;
    if (algo == 0u) {
        color = volume_dvr(ray_start, step_in_dir);
    } else if (algo == 1u) {
        color = mip(ray_start, step_in_dir);
    } else if (algo == 2u) {
        color = minip(ray_start, step_in_dir);
    } else if (algo == 3u) {
        color = average_projection(ray_start, step_in_dir);
    } else if (algo == 4u) {
        color = label_iso(ray_start, step_in_dir);
    } else if (algo == 5u) {
        color = absorptionrgba(ray_start, step_in_dir);
    } else if (algo == 6u) {
        color = additivergba(ray_start, step_in_dir);
    } else {
        color = volumeindexedrgba(ray_start, step_in_dir);
    }

    if (color.a <= 0.001) {
        discard;
    }

    return color;
}
