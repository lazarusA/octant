//! The orbit camera: its uniform, rays and projections agree, and panning
//! moves the picture with the pointer.

use super::camera::{FOV_SCALE, OrbitCamera};

fn camera(target: [f32; 3]) -> OrbitCamera {
    OrbitCamera {
        rotation_x: 0.4,
        rotation_y: -0.7,
        distance: 1.8,
        target,
        aspect: 1.5,
    }
}

const POINTS: [[f32; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [0.3, -0.2, 0.5],
    [-0.8, 0.4, -0.1],
    [0.1, 0.6, 0.2],
];

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
}

/// Applies the uniform's column-major view matrix as the shader does.
fn shader_view(cam: &OrbitCamera, p: [f32; 3]) -> [f32; 3] {
    let m = cam.uniform().view;
    std::array::from_fn(|r| m[0][r] * p[0] + m[1][r] * p[1] + m[2][r] * p[2] + m[3][r])
}

#[test]
fn uniform_view_matches_cpu_view() {
    let cam = camera([0.2, -0.1, 0.3]);
    for p in POINTS {
        assert!(close(shader_view(&cam, p), cam.to_view(p)), "{p:?}");
    }
    assert!(close(cam.to_view(cam.eye()), [0.0; 3]));
    assert!(close(cam.uniform().eye, cam.eye()));
}

/// Without a pan, the eye sits where the camera always did: `distance`
/// from the origin, turned by the rotations.
#[test]
fn centered_eye_matches_the_orbit() {
    let cam = camera([0.0; 3]);
    let (sy, cy) = cam.rotation_y.sin_cos();
    let (sx, cx) = cam.rotation_x.sin_cos();
    let d = cam.distance;
    assert!(close(cam.eye(), [-sy * cx * d, sx * d, cy * cx * d]));
}

#[test]
fn rays_pass_through_projected_points() {
    let cam = camera([0.1, 0.2, -0.3]);
    for p in POINTS {
        let clip = cam.project(p).expect("in front of the camera");
        let (origin, dir) = cam.ray(clip);
        let to_p = [p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]];
        let t = to_p[0] * dir[0] + to_p[1] * dir[1] + to_p[2] * dir[2];
        let hit = [
            origin[0] + dir[0] * t,
            origin[1] + dir[1] * t,
            origin[2] + dir[2] * t,
        ];
        assert!(close(hit, p), "{p:?}: {hit:?}");
    }
}

#[test]
fn target_projects_to_the_center() {
    let target = [0.4, 0.1, -0.2];
    let clip = camera(target).project(target).expect("visible");
    assert!(clip[0].abs() < 1e-5 && clip[1].abs() < 1e-5);
}

/// Moving the target by `-screen_offset(delta)` moves what was at the
/// target `delta` screen points (y down).
#[test]
fn screen_offset_follows_the_pointer() {
    let height = 600.0;
    let width = height * 1.5;
    let cam = camera([0.0; 3]);
    let delta = [30.0, -45.0];
    let offset = cam.screen_offset(delta, height);
    let moved = OrbitCamera {
        target: [-offset[0], -offset[1], -offset[2]],
        ..cam
    };
    let clip = moved.project([0.0; 3]).expect("visible");
    let screen = [clip[0] * width / 2.0, -clip[1] * height / 2.0];
    assert!((screen[0] - delta[0]).abs() < 1e-3, "{screen:?}");
    assert!((screen[1] - delta[1]).abs() < 1e-3, "{screen:?}");
}

#[test]
fn points_behind_the_camera_do_not_project() {
    let cam = camera([0.0; 3]);
    let eye = cam.eye();
    let behind = [eye[0] * 1.5, eye[1] * 1.5, eye[2] * 1.5];
    assert!(cam.project(behind).is_none());
    assert_eq!(cam.uniform().fov_scale, FOV_SCALE);
}
