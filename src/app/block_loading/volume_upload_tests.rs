//! What a point cloud's changed planes upload: the volume's own planes at
//! their place, or a thinned cloud's thinned planes at theirs.

use super::volume_upload::point_cloud_planes;
use crate::app::layers::PointGrid;
use crate::data::VolumeData;

/// 5 x 4 x 3 voxels numbered in order.
fn volume() -> VolumeData {
    let values = (0..60).map(|i| i as f32).collect();
    VolumeData::new(5, 4, 3, values, 0.0, 59.0, String::new())
}

#[test]
fn changed_planes_upload_where_the_cloud_draws_them() {
    let v = volume();
    let mut scratch = Vec::new();
    let (offset, planes) = point_cloud_planes(&v, None, 1..2, &mut scratch);
    assert_eq!(offset, 20);
    assert_eq!(planes, &v.values[20..40]);
    // Thinned by 2: 3 x 2 cells a plane, plane 2 at 2 x 6.
    let grid = PointGrid {
        stride: 2,
        width: 3,
        height: 2,
    };
    let (offset, planes) = point_cloud_planes(&v, Some(&grid), 2..3, &mut scratch);
    assert_eq!(offset, 12);
    assert_eq!(planes, &[46.0, 48.0, 49.0, 56.0, 58.0, 59.0]);
    assert_eq!(planes, v.thinned_planes(2, 2..3), "as the cloud was built");
}
