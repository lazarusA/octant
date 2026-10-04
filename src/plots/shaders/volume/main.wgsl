// Fragment stage: ray setup against the box and clip planes, then mode dispatch.

// Upper bound on samples per ray (a 2048-voxel diagonal at 2 samples per voxel).
const MAX_SAMPLES: i32 = 4096;

struct ClipResult {
    clipped: bool,
    p1: vec3<f32>,
    p2: vec3<f32>,
};

fn process_clip_planes(p1_in: vec3<f32>, p2_in: vec3<f32>) -> ClipResult {
    var p1 = p1_in;
    var p2 = p2_in;
    let count = min(uniforms.num_clip_planes, 8u);

    for (var i = 0u; i < count; i = i + 1u) {
        let plane = uniforms.clip_planes[i];
        let d1 = dot(p1, plane.xyz) - plane.w;
        let d2 = dot(p2, plane.xyz) - plane.w;
        if (d1 < 0.0 && d2 < 0.0) {
            return ClipResult(true, p1, p1);
        } else if (d1 < 0.0) {
            p1 = p1 - d1 * (p2 - p1) / (d2 - d1);
        } else if (d2 < 0.0) {
            p2 = p2 - d2 * (p1 - p2) / (d1 - d2);
        }
    }
    return ClipResult(false, p1, p2);
}

// Interleaved gradient noise in [0, 1): a per-pixel start offset that turns
// wood-grain banding into fine, unstructured grain.
fn pixel_jitter(frag: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(dot(frag, vec2<f32>(0.06711056, 0.00583715))));
}

// Samples for the segment p1..p2: `quality` per voxel crossed (at least 8),
// starting a jittered fraction of a step in.
fn build_ray(p1: vec3<f32>, p2: vec3<f32>, frag: vec2<f32>) -> Ray {
    let segment = p2 - p1;
    let voxels = length(segment * volume_dims());
    let count = clamp(i32(ceil(voxels * max(uniforms.quality, 0.05))), 8, MAX_SAMPLES);
    let step = segment / f32(count);
    let world_scale = vec3<f32>(uniforms.aspect_x, uniforms.aspect_y, uniforms.aspect_z);
    return Ray(p1 + pixel_jitter(frag) * step, step, count, length(step * world_scale));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cy = cos(-uniforms.rotation_y);
    let sy = sin(-uniforms.rotation_y);
    let cx = cos(-uniforms.rotation_x);
    let sx = sin(-uniforms.rotation_x);

    // Same camera distance as `vs_main`, so rays pass through the rasterized box.
    let eye_cam = vec3<f32>(0.0, 0.0, camera_distance());
    let eye_x_rot = vec3<f32>(eye_cam.x, cx * eye_cam.y - sx * eye_cam.z, sx * eye_cam.y + cx * eye_cam.z);
    let eye_rot = vec3<f32>(cy * eye_x_rot.x + sy * eye_x_rot.z, eye_x_rot.y, -sy * eye_x_rot.x + cy * eye_x_rot.z);

    let scale_vec = max(vec3<f32>(uniforms.aspect_x, uniforms.aspect_y, uniforms.aspect_z), vec3<f32>(0.001));
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

    let clip_res = process_clip_planes(eye_unit + max(t_enter, 0.0) * dir, eye_unit + t_exit * dir);
    if (clip_res.clipped) {
        discard;
    }

    let ray = build_ray(clip_res.p1, clip_res.p2, in.position.xy);
    let algo = uniforms.algorithm;
    var color: vec4<f32>;
    if (algo == 0u) {
        color = volume_dvr(ray);
    } else if (algo == 1u) {
        color = mip(ray);
    } else if (algo == 2u) {
        color = minip(ray);
    } else if (algo == 3u) {
        color = average_projection(ray);
    } else if (algo == 4u) {
        color = label_iso(ray);
    } else if (algo == 5u) {
        color = absorptionrgba(ray);
    } else if (algo == 6u) {
        color = additivergba(ray);
    } else {
        color = volumeindexedrgba(ray);
    }

    if (color.a <= 0.001) {
        discard;
    }
    // Every mode returns premultiplied color (see the pipeline's blend state).
    return color;
}
