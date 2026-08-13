// Nothing type, to encode if some variable doesn't contain any data
struct Nothing { // Nothing type, to encode if some variable doesn't contain any data
    empty: bool, // empty structs are not allowed
};

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
    depth_shift: f32,
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
    _pad0: u32,
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

    let cam_dist = clamp(uniforms.zoom, 1.1, 10.0);
    let cam_z = pos_rot.z - cam_dist;
    let dist_positive = max(-cam_z, 0.1);

    let fov_scale = 1.6;
    let screen_asp = max(uniforms.screen_aspect, 0.1);
    let proj_x = (pos_rot.x * fov_scale) / screen_asp;
    let proj_y = pos_rot.y * fov_scale;

    let z_near = 0.5;
    let z_far = 30.0;
    let proj_z = (z_far / (z_far - z_near)) * dist_positive - (z_far * z_near / (z_far - z_near));

    out.position = vec4<f32>(proj_x, proj_y, proj_z, dist_positive);
    return out;
}

fn is_nan(val: f32) -> bool {
    return val != val;
}

fn no_solution(x: f32) -> bool {
    return abs(x) < 0.0001 || is_nan(x) || x > 1e20 || x < -1e20;
}

fn get_lowclip_color() -> vec4<f32> {
    return uniforms.color.lowclip_color;
}

fn get_highclip_color() -> vec4<f32> {
    return uniforms.color.highclip_color;
}

fn get_nan_color() -> vec4<f32> {
    return uniforms.color.nan_color;
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
    let base_x = u32(texCoord.x * f32(grid_w - 1u));
    let base_y = u32(norm_y * f32(grid_h - 1u));
    let base_z = u32(texCoord.z * f32(grid_d - 1u));

    let gx = (base_x + uniforms.shift_x) % grid_w;
    let gy = (base_y + uniforms.shift_y) % grid_h;
    let gz = (base_z + uniforms.shift_z) % grid_d;

    let idx = min(gz * grid_h * grid_w + gy * grid_w + gx, total_len - 1u);
    return data_buffer[idx];
}

fn sample_volume_rgba(pos: vec3<f32>) -> vec4<f32> {
    let s = sample_volume_scalar(pos);
    return evaluate_plot_color(s, uniforms.color);
}

fn gennormal(uvw: vec3<f32>) -> vec3<f32> {
    let grid_w = f32(max(uniforms.width, 1u));
    let grid_h = f32(max(uniforms.height, 1u));
    let grid_d = f32(max(uniforms.depth, 1u));

    let dx = vec3<f32>(1.0 / grid_w, 0.0, 0.0);
    let dy = vec3<f32>(0.0, 1.0 / grid_h, 0.0);
    let dz = vec3<f32>(0.0, 0.0, 1.0 / grid_d);

    let p_x0 = clamp(uvw - dx, vec3<f32>(0.0), vec3<f32>(1.0));
    let p_x1 = clamp(uvw + dx, vec3<f32>(0.0), vec3<f32>(1.0));
    let p_y0 = clamp(uvw - dy, vec3<f32>(0.0), vec3<f32>(1.0));
    let p_y1 = clamp(uvw + dy, vec3<f32>(0.0), vec3<f32>(1.0));
    let p_z0 = clamp(uvw - dz, vec3<f32>(0.0), vec3<f32>(1.0));
    let p_z1 = clamp(uvw + dz, vec3<f32>(0.0), vec3<f32>(1.0));

    let nx = sample_volume_scalar(p_x0) - sample_volume_scalar(p_x1);
    let ny = sample_volume_scalar(p_y0) - sample_volume_scalar(p_y1);
    let nz = sample_volume_scalar(p_z0) - sample_volume_scalar(p_z1);

    let grad = vec3<f32>(nx, ny, nz);
    let len = length(grad);
    if (len < 0.00001) {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return normalize(grad);
}

fn smooth_zero_max(x: f32) -> f32 {
    let c: f32 = 0.00390625;
    let xswap: f32 = 0.6406707120152759;
    let yswap: f32 = 0.20508383900190955;
    let shift: f32 = 1.0 + xswap - yswap;
    var pow8: f32 = x + shift;
    pow8 = pow8 * pow8;
    pow8 = pow8 * pow8;
    pow8 = pow8 * pow8;
    if (x < yswap) {
        return c * pow8;
    } else {
        return x;
    }
}

fn blinnphong(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, color: vec3<f32>) -> vec3<f32> {
    let light_dir = normalize(vec3<f32>(0.4, 0.8, 0.6));
    let diff_coeff = max(dot(light_dir, N), 0.0) + max(dot(light_dir, -N), 0.0) * 0.4;
    let H = normalize(light_dir + V);
    let spec_coeff = pow(max(dot(H, N), 0.0), uniforms.shininess);

    let ambient = max(uniforms.ambient, vec3<f32>(0.35));
    return ambient * color + uniforms.diffuse * diff_coeff * color + uniforms.light_color * uniforms.specular * spec_coeff;
}

fn hitBox(orig: vec3<f32>, dir: vec3<f32>, scale_vec: vec3<f32>) -> vec2<f32> {
    let box_min = -(scale_vec * 0.5);
    let box_max = scale_vec * 0.5;
    let inv_dir = 1.0 / dir;
    let tmin_tmp = (box_min - orig) * inv_dir;
    let tmax_tmp = (box_max - orig) * inv_dir;
    let tmin = min(tmin_tmp, tmax_tmp);
    let tmax = max(tmin_tmp, tmax_tmp);
    let t0 = max(tmin.x, max(tmin.y, tmin.z));
    let t1 = min(tmax.x, min(tmax.y, tmax.z));
    return vec2<f32>(t0, t1);
}

// 0. Default Mode: Fast HitBox Threshold Volume Raymarching
fn volume_hitbox_threshold(vOrigin: vec3<f32>, rayDir: vec3<f32>, bounds: vec2<f32>, scale_vec: vec3<f32>) -> vec4<f32> {
    let safe_dir = max(abs(rayDir), vec3<f32>(0.0001));
    let inc = 1.0 / safe_dir;
    var delta = min(inc.x, min(inc.y, inc.z));
    let samples_count = i32(max(uniforms.samples, 8u));
    delta = delta / f32(samples_count);

    var accumColor = vec3<f32>(0.0);
    var alphaAcc: f32 = 0.0;

    let threshold_min = uniforms.color.cmin;
    let threshold_max = uniforms.color.cmax;

    var t = bounds.x;
    for (var i = 0; i < samples_count; i = i + 1) {
        if (t >= bounds.y) {
            break;
        }
        let p = vOrigin + rayDir * t;
        var texCoord = p / scale_vec + vec3<f32>(0.5);

        let epsilon: f32 = 0.000001;
        texCoord = clamp(texCoord, vec3<f32>(0.0), vec3<f32>(1.0 - epsilon));

        let d = sample_volume_scalar(texCoord);

        let is_nan_sample = (d != d || abs(d) > 1e30);
        let is_low_sample = (d < threshold_min) && !is_nan_sample;
        let is_high_sample = (d > threshold_max) && !is_nan_sample;
        let is_in_bounds = (d >= threshold_min && d <= threshold_max) && !is_nan_sample;

        var col: vec3<f32> = vec3<f32>(0.0);
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
            let range = max(threshold_max - threshold_min, 0.0001);
            let sampLoc = clamp((d - threshold_min) / range, 0.0, 1.0);
            col = sample_colormap(uniforms.color.colormap, sampLoc);
            let alpha_exponent = max(uniforms.absorption, 0.1);
            alpha = clamp(pow(max(sampLoc, 0.001), 1.0 / alpha_exponent), 0.01, 1.0);
        }

        if (alpha > 0.0) {
            accumColor = accumColor + (1.0 - alphaAcc) * alpha * col;
            alphaAcc = alphaAcc + alpha * (1.0 - alphaAcc);

            if (alphaAcc >= 0.99) {
                break;
            }
        }
        t = t + delta;
    }

    return vec4<f32>(accumColor, alphaAcc);
}

// Modes (Isosurface, MIP, Absorption, Additive, Indexed, Contours)
fn isosurface(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front;
    var c = vec4<f32>(0.0);
    let diffuse_color = color_lookup(uniforms.isovalue);
    let camdir = normalize(-dir);
    let step_size = length(dir);
    let samples_count = i32(max(uniforms.samples, 8u));

    for (var i = 0; i < samples_count; i = i + 1) {
        let density = sample_volume_scalar(pos);
        if (abs(density - uniforms.isovalue) < uniforms.isorange) {
            let N = gennormal(pos);
            let L = uniforms.light_direction;
            c = vec4<f32>(
                blinnphong(N, camdir, L, diffuse_color.rgb),
                diffuse_color.a
            );
            break;
        }
        pos = pos + dir;
    }
    return c;
}

fn mip(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front + dir;
    var maximum: f32 = -1e30;
    let highclip_visible = uniforms.color.highclip_color.a > 0.0;
    let samples_count = i32(max(uniforms.samples, 8u));

    for (var i = 0; i < samples_count; i = i + 1) {
        let density = sample_volume_scalar(pos);
        let consider_sample = (density < uniforms.color.cmax) || highclip_visible;
        if (consider_sample && (maximum < density)) {
            maximum = density;
        }
        pos = pos + dir;
    }
    if (maximum == -1e30) {
        maximum = 1e30;
    }
    return color_lookup(maximum);
}

fn absorptionrgba(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front;
    var transmittance: f32 = 1.0;
    var color_sum = vec3<f32>(0.0);
    let step_size = length(dir);
    let samples_count = i32(max(uniforms.samples, 8u));

    for (var i = 0; i < samples_count; i = i + 1) {
        let color_sample = sample_volume_rgba(pos);

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

fn contours(front: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    var pos = front;
    var transmittance: f32 = 1.0;
    var color_sum = vec3<f32>(0.0);
    let camdir = normalize(-dir);
    let step_size = length(dir);
    let samples_count = i32(max(uniforms.samples, 8u));

    for (var i = 0; i < samples_count; i = i + 1) {
        let intensity = sample_volume_scalar(pos);
        if (intensity >= uniforms.color.cmin && intensity <= uniforms.color.cmax) {
            let color_sample = color_lookup(intensity);
            let opacity = clamp(step_size * max(color_sample.a, 0.1) * uniforms.absorption, 0.0, 1.0);

            let N = gennormal(pos);
            let L = normalize(vec3<f32>(0.4, 0.8, 0.6));
            let opaque = blinnphong(N, camdir, L, color_sample.rgb);
            color_sum = color_sum + (transmittance * opacity) * opaque;
            transmittance = transmittance * (1.0 - opacity);

            if (transmittance <= 0.01) {
                break;
            }
        }
        pos = pos + dir;
    }
    if (1.0 - transmittance <= 0.0) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(color_sum / max(1.0 - transmittance, 0.001), 1.0 - transmittance);
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
    var color: vec4<f32>;

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

    let algo = uniforms.algorithm;

    if (algo == 0u) {
        // Mode 0: Default HitBox Threshold Volume Raymarching
        let vOrigin = eye_rot;
        let vDirection = normalize(in.frag_vert - eye_rot);
        let rayDir = normalize(vDirection);
        var bounds = hitBox(vOrigin, rayDir, scale_vec);
        if (bounds.x > bounds.y) {
            discard;
        }
        bounds.x = max(bounds.x, 0.0);
        color = volume_hitbox_threshold(vOrigin, rayDir, bounds, scale_vec);
    } else {
        // Modes (Isosurface, MIP, Absorption, Additive, Indexed, Contours)
        let eye_unit = vec3<f32>(0.5) + eye_rot / scale_vec;
        let back_position = in.frag_vert / scale_vec + vec3<f32>(0.5);
        let dir = normalize(back_position - eye_unit);

        let is_outside_box = (
            eye_unit.x < 0.0 || eye_unit.y < 0.0 || eye_unit.z < 0.0 ||
            eye_unit.x > 1.0 || eye_unit.y > 1.0 || eye_unit.z > 1.0
        );

        if ((dir.x == 0.0 && dir.y == 0.0 && dir.z == 0.0) || is_nan(dir.x) || is_nan(dir.y) || is_nan(dir.z)) {
            discard;
        }

        let solution_1 = (vec3<f32>(1.0) - eye_unit) / dir;
        let solution_0 = (vec3<f32>(0.0) - eye_unit) / dir;

        var solutions_min = min(solution_0, solution_1);
        var solutions_max = max(solution_0, solution_1);

        let typemax: f32 = 1e30;
        if (no_solution(solutions_min.x)) { solutions_min.x = -typemax; }
        if (no_solution(solutions_min.y)) { solutions_min.y = -typemax; }
        if (no_solution(solutions_min.z)) { solutions_min.z = -typemax; }

        if (no_solution(solutions_max.x)) { solutions_max.x = typemax; }
        if (no_solution(solutions_max.y)) { solutions_max.y = typemax; }
        if (no_solution(solutions_max.z)) { solutions_max.z = typemax; }

        let start_solution = max(max(solutions_min.x, solutions_min.y), solutions_min.z);
        let stop_solution = min(min(solutions_max.x, solutions_max.y), solutions_max.z);

        if (stop_solution < max(start_solution, 0.0)) {
            discard;
        }

        let start = eye_unit + select(0.0, start_solution, is_outside_box) * dir;
        let stop = eye_unit + stop_solution * dir;

        let clip_res = process_clip_planes(start, stop);
        if (clip_res.clipped) {
            discard;
        }

        let step_in_dir = (clip_res.p2 - clip_res.p1) / f32(max(uniforms.samples, 1u));
        let ray_start = clip_res.p1;

        if (algo == 1u) {
            color = isosurface(ray_start, step_in_dir);
        } else if (algo == 2u) {
            color = mip(ray_start, step_in_dir);
        } else if (algo == 3u) {
            color = absorptionrgba(ray_start, step_in_dir);
        } else if (algo == 4u) {
            color = additivergba(ray_start, step_in_dir);
        } else if (algo == 5u) {
            color = volumeindexedrgba(ray_start, step_in_dir);
        } else {
            color = contours(ray_start, step_in_dir);
        }
    }

    if (color.a <= 0.001) {
        discard;
    }

    return color;
}
