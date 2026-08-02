/// 3D Spherical / Globe Projection Plot.
///
/// Maps lat-lon planetary datasets (e.g. Earth system data cubes) onto a 3D sphere.
pub struct SpherePlot {
    pub radius: f32,
}

impl SpherePlot {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}
