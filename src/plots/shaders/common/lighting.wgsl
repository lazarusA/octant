// Shared 3D directional lighting and screen-space normal calculation utilities

fn evaluate_directional_lighting(
    normal: vec3<f32>,
    light_dir: vec3<f32>,
    ambient: f32,
    diffuse: f32,
) -> f32 {
    let n = normalize(normal);
    let l = normalize(light_dir);
    let n_dot_l = abs(dot(n, l));
    return ambient + diffuse * n_dot_l;
}

fn compute_screen_space_normal(world_pos: vec3<f32>) -> vec3<f32> {
    let dpx = dpdx(world_pos);
    let dpy = dpdy(world_pos);
    let cross_norm = cross(dpx, dpy);
    if (dot(cross_norm, cross_norm) > 1e-6) {
        return normalize(-cross_norm);
    }
    return vec3<f32>(0.0, 1.0, 0.0);
}
