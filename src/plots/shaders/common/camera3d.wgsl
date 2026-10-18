// Shared 3D camera of Octant's WGSL shaders, built on the CPU by
// `src/plots/camera.rs` `OrbitCamera::uniform`.

// Mirrors `CameraUniform`: view space has the camera at the origin looking
// down -z.
struct Camera {
    // Object to view space.
    view: mat4x4<f32>,
    // The camera's position in object space.
    eye: vec3<f32>,
    fov_scale: f32,
    // Screen width over height.
    aspect: f32,
    near: f32,
    far: f32,
    _pad: f32,
};

/// Object space position in view space.
fn camera_view(cam: Camera, pos: vec3<f32>) -> vec3<f32> {
    return (cam.view * vec4<f32>(pos, 1.0)).xyz;
}

/// Object space direction in view space.
fn camera_view_normal(cam: Camera, norm: vec3<f32>) -> vec3<f32> {
    return normalize((cam.view * vec4<f32>(norm, 0.0)).xyz);
}

/// View space direction in object space.
fn camera_to_object_dir(cam: Camera, v: vec3<f32>) -> vec3<f32> {
    let rot = mat3x3<f32>(cam.view[0].xyz, cam.view[1].xyz, cam.view[2].xyz);
    return transpose(rot) * v;
}

/// Perspective projection of a view space position, with depth growing
/// linearly in 1/distance between the camera's near and far planes
/// (`oit_accumulate` inverts it).
fn camera_project(cam: Camera, view_pos: vec3<f32>) -> vec4<f32> {
    let dist = max(-view_pos.z, 1e-5);
    let proj_x = (view_pos.x * cam.fov_scale) / cam.aspect;
    let proj_y = view_pos.y * cam.fov_scale;
    let proj_z = (cam.far / (cam.far - cam.near)) * dist - (cam.far * cam.near / (cam.far - cam.near));
    return vec4<f32>(proj_x, proj_y, proj_z, dist);
}
