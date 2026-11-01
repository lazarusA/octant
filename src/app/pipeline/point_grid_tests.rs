//! Point clouds of more voxels than the canvas budget draw a grid thinned
//! along x and y, with a GPU.

use crate::app::OctantApp;
use crate::app::layers::{LayerId, PointGrid};
use crate::app::test_support::render_state;
use crate::data::VolumeData;
use crate::plots::PlotType;
use crate::ui::hover::raycast_volume::VolumeSampler;
use std::sync::atomic::Ordering;

const SIZE: (usize, usize, usize) = (200, 100, 40);

/// A 200 x 100 x 40 point cloud built on a 512 x 512 canvas (262 144 points
/// at most: 6 553 a plane).
fn built() -> Option<OctantApp> {
    let mut app = OctantApp {
        wgpu_render_state: Some(render_state()?),
        ..Default::default()
    };
    app.selected.plot_type = PlotType::PointCloud;
    app.layout.canvas_px = [512, 512];
    let (w, h, d) = SIZE;
    let values = (0..w * h * d).map(|i| i as f32).collect();
    let volume = VolumeData::new(w, h, d, values, 0.0, (w * h * d) as f32, "v".into());
    app.rebuild_pipeline_with_volume_data(LayerId::BASE, volume);
    Some(app)
}

fn instances(app: &OctantApp) -> Option<u32> {
    let r = app.layers.base.renderers.point_cloud.as_ref()?;
    Some(r.instance_count.load(Ordering::Relaxed))
}

#[test]
fn large_volumes_draw_thinned_point_clouds() {
    let Some(mut app) = built() else {
        eprintln!("SKIPPED large_volumes_draw_thinned_point_clouds: no GPU device");
        return;
    };
    let grid = PointGrid {
        stride: 2,
        width: 100,
        height: 50,
    };
    assert_eq!(app.layers.base.data.points, Some(grid));
    assert_eq!(instances(&app), Some(100 * 50 * 40), "every plane, thinned");
    let params = app.get_point_cloud_uniform_params(&app.layers.base, 1.0);
    assert_eq!((params.width, params.height), (100, 50));
    // The hover picks the drawn cells and reads the voxels they draw.
    let Some(sampler) = VolumeSampler::from_app(&app, None) else {
        panic!("a volume");
    };
    assert_eq!(sampler.drawn_cell(5, 7), ((2, 3), (100, 50)));
    assert_eq!(sampler.width, 200, "the hover reads every voxel");

    // A canvas showing every voxel draws them all.
    app.layout.canvas_px = [4000, 4000];
    app.rethin_points();
    assert_eq!(app.layers.base.data.points, None);
    assert_eq!(instances(&app), Some(200 * 100 * 40));
}
