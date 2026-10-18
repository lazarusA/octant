//! The orbit camera of the 3D plots (surface, sphere, volume, point cloud and
//! their coastlines): one CPU model that builds the shaders' `Camera` uniform
//! (`shaders/common/camera3d.wgsl`) and the hover's rays and projections, so
//! what is drawn and what is picked never disagree.
//!
//! The camera looks at `target` from `distance` away, turned by `rotation_y`
//! (about the vertical axis) then `rotation_x` (pitch). View space has the
//! camera at the origin looking down -z; clip x and y are `fov_scale` times
//! the view position over its distance, x divided by the screen aspect.

use crate::plots::PlotType;
use bytemuck::{Pod, Zeroable};

/// Perspective scale: clip coordinate per view unit at unit distance.
pub const FOV_SCALE: f32 = 1.6;
/// Farthest the camera may move from its target.
pub const MAX_DISTANCE: f32 = 8.0;
/// Far plane; the scene stays within a few units of the origin.
const FAR: f32 = 50.0;
/// Farthest a panned target may move from the scene's center, per axis.
pub const TARGET_LIMIT: f32 = 5.0;

/// Closest the camera may come to its target: the sphere keeps above its
/// unit globe, other plots may come close to any point they are panned to.
pub fn min_distance(plot_type: PlotType) -> f32 {
    if plot_type == PlotType::Sphere {
        1.02
    } else {
        0.02
    }
}

/// Mirrors WGSL `Camera` in `shaders/common/camera3d.wgsl`.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct CameraUniform {
    /// Object to view space, column-major.
    pub view: [[f32; 4]; 4],
    /// The camera's position in object space.
    pub eye: [f32; 3],
    pub fov_scale: f32,
    /// Screen width over height.
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub _pad: f32,
}

/// A 3D plot's camera for one frame (zero-allocation `Copy`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OrbitCamera {
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub distance: f32,
    pub target: [f32; 3],
    /// Screen width over height.
    pub aspect: f32,
}

impl Default for OrbitCamera {
    /// The view a 3D plot opens with.
    fn default() -> Self {
        Self {
            rotation_x: 0.25,
            rotation_y: 0.0,
            distance: 2.5,
            target: [0.0; 3],
            aspect: 1.0,
        }
    }
}

impl OrbitCamera {
    /// The rotation's rows: object space directions to view space.
    fn rows(&self) -> [[f32; 3]; 3] {
        let (sy, cy) = self.rotation_y.sin_cos();
        let (sx, cx) = self.rotation_x.sin_cos();
        [
            [cy, 0.0, sy],
            [sx * sy, cx, -sx * cy],
            [-cx * sy, sx, cx * cy],
        ]
    }

    /// Near plane: close enough for the nearest the camera comes.
    pub fn near(&self) -> f32 {
        (self.distance * 0.05).clamp(1e-4, 0.01)
    }

    /// Object space position in view space.
    pub fn to_view(&self, p: [f32; 3]) -> [f32; 3] {
        let d = sub(p, self.target);
        let [r0, r1, r2] = self.rows();
        [dot(r0, d), dot(r1, d), dot(r2, d) - self.distance]
    }

    /// View space direction in object space.
    pub fn to_object_dir(&self, v: [f32; 3]) -> [f32; 3] {
        let [r0, r1, r2] = self.rows();
        [
            r0[0] * v[0] + r1[0] * v[1] + r2[0] * v[2],
            r0[1] * v[0] + r1[1] * v[1] + r2[1] * v[2],
            r0[2] * v[0] + r1[2] * v[1] + r2[2] * v[2],
        ]
    }

    /// The camera's position in object space.
    pub fn eye(&self) -> [f32; 3] {
        add(self.target, self.to_object_dir([0.0, 0.0, self.distance]))
    }

    /// Clip x and y (-1..1, y up) of an object space point, `None` behind
    /// the near plane.
    pub fn project(&self, p: [f32; 3]) -> Option<[f32; 2]> {
        let v = self.to_view(p);
        let dist = -v[2];
        (dist > self.near()).then(|| {
            [
                v[0] * FOV_SCALE / (self.aspect * dist),
                v[1] * FOV_SCALE / dist,
            ]
        })
    }

    /// The ray through clip position `clip` (y up): origin at the eye and
    /// unit direction, in object space.
    pub fn ray(&self, clip: [f32; 2]) -> ([f32; 3], [f32; 3]) {
        let v = [clip[0] * self.aspect / FOV_SCALE, clip[1] / FOV_SCALE, -1.0];
        (self.eye(), normalize(self.to_object_dir(v)))
    }

    /// Object space offset that moves the picture by `delta` screen points
    /// on a canvas `height` points tall, for points at the target's depth.
    pub fn screen_offset(&self, delta: [f32; 2], height: f32) -> [f32; 3] {
        let per_point = 2.0 * self.distance / (FOV_SCALE * height.max(1.0));
        self.to_object_dir([delta[0] * per_point, -delta[1] * per_point, 0.0])
    }

    /// The uniform the shaders read.
    pub fn uniform(&self) -> CameraUniform {
        let [r0, r1, r2] = self.rows();
        let t = self.to_view([0.0; 3]);
        CameraUniform {
            view: [
                [r0[0], r1[0], r2[0], 0.0],
                [r0[1], r1[1], r2[1], 0.0],
                [r0[2], r1[2], r2[2], 0.0],
                [t[0], t[1], t[2], 1.0],
            ],
            eye: self.eye(),
            fov_scale: FOV_SCALE,
            aspect: self.aspect.max(0.01),
            near: self.near(),
            far: FAR,
            _pad: 0.0,
        }
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = dot(v, v).sqrt().max(1e-12);
    [v[0] / len, v[1] / len, v[2] / len]
}
