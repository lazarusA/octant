// Shared 3D camera projection, transformation, and lighting routines for Octant WGSL shaders

/// Rigid 3D rotation of a position around Y and X camera axes.
fn rotate_camera_yx(pos: vec3<f32>, rot_y: f32, rot_x: f32) -> vec3<f32> {
    let cy = cos(rot_y);
    let sy = sin(rot_y);
    let cx = cos(rot_x);
    let sx = sin(rot_x);

    // Y-axis rotation
    let pos_y = vec3<f32>(
        cy * pos.x + sy * pos.z,
        pos.y,
        -sy * pos.x + cy * pos.z
    );

    // X-axis rotation
    return vec3<f32>(
        pos_y.x,
        cx * pos_y.y - sx * pos_y.z,
        sx * pos_y.y + cx * pos_y.z
    );
}

/// Rigid 3D rotation of a normal vector around Y and X camera axes.
fn rotate_normal_yx(norm: vec3<f32>, rot_y: f32, rot_x: f32) -> vec3<f32> {
    let cy = cos(rot_y);
    let sy = sin(rot_y);
    let cx = cos(rot_x);
    let sx = sin(rot_x);

    let norm_y = vec3<f32>(
        cy * norm.x + sy * norm.z,
        norm.y,
        -sy * norm.x + cy * norm.z
    );

    return normalize(vec3<f32>(
        norm_y.x,
        cx * norm_y.y - sx * norm_y.z,
        sx * norm_y.y + cx * norm_y.z
    ));
}

/// Perspective projection transformation and linear depth calculation.
fn project_perspective(
    pos_rot: vec3<f32>,
    aspect_ratio: f32,
    zoom: f32,
    fov_scale: f32,
    min_dist: f32,
) -> vec4<f32> {
    let cam_dist = clamp(zoom, min_dist, 10.0);
    let cam_z = pos_rot.z - cam_dist;
    let dist_positive = max(-cam_z, 0.001);

    let proj_x = (pos_rot.x * fov_scale) / aspect_ratio;
    let proj_y = pos_rot.y * fov_scale;

    let z_near = 0.01;
    let z_far = 50.0;
    let proj_z = (z_far / (z_far - z_near)) * dist_positive - (z_far * z_near / (z_far - z_near));

    return vec4<f32>(proj_x, proj_y, proj_z, dist_positive);
}

/// Evaluates 3D directional lighting with two-sided support for surface meshes.
fn evaluate_directional_lighting(
    geom_normal: vec3<f32>,
    light_dir: vec3<f32>,
    ambient: f32,
    diffuse_scale: f32,
) -> f32 {
    let diffuse = max(abs(dot(geom_normal, normalize(light_dir))), 0.25);
    return clamp(ambient + diffuse * diffuse_scale, 0.3, 1.0);
}
