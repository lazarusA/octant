use octant::data::{AggregationOp, MatrixPyramid, ViewportResampler};

#[test]
fn test_pyramid_levels_and_dimensions() {
    let width = 2048;
    let height = 1024;
    let values: Vec<f32> = (0..width * height).map(|i| (i % 100) as f32).collect();

    let pyramid = MatrixPyramid::new(
        &values,
        width,
        height,
        "test_data",
        AggregationOp::Mean,
        512,
    );

    assert!(pyramid.levels.len() >= 3);
    assert_eq!(pyramid.levels[0].width, 2048);
    assert_eq!(pyramid.levels[0].height, 1024);
    assert_eq!(pyramid.levels[0].level_idx, 0);

    assert_eq!(pyramid.levels[1].width, 1024);
    assert_eq!(pyramid.levels[1].height, 512);
    assert_eq!(pyramid.levels[1].scale_x, 2.0);

    assert_eq!(pyramid.levels[2].width, 512);
    assert_eq!(pyramid.levels[2].height, 256);
    assert_eq!(pyramid.levels[2].scale_x, 4.0);
}

#[test]
fn test_pyramid_downsampling_aggregation_ops() {
    // 2x2 grid:
    // [10.0, 20.0]
    // [30.0, 40.0]
    let src = vec![10.0, 20.0, 30.0, 40.0];

    let mean_pyr = MatrixPyramid::new(&src, 2, 2, "mean", AggregationOp::Mean, 1);
    let max_pyr = MatrixPyramid::new(&src, 2, 2, "max", AggregationOp::Max, 1);
    let min_pyr = MatrixPyramid::new(&src, 2, 2, "min", AggregationOp::Min, 1);
    let nearest_pyr = MatrixPyramid::new(&src, 2, 2, "nearest", AggregationOp::Nearest, 1);

    assert_eq!(mean_pyr.levels.last().unwrap().values[0], 25.0);
    assert_eq!(max_pyr.levels.last().unwrap().values[0], 40.0);
    assert_eq!(min_pyr.levels.last().unwrap().values[0], 10.0);
    assert_eq!(nearest_pyr.levels.last().unwrap().values[0], 10.0);
}

#[test]
fn test_pyramid_nan_handling() {
    // 2x2 grid with 2 NaNs and 2 numbers:
    // [f32::NAN, 20.0]
    // [40.0, f32::NAN]
    let src = vec![f32::NAN, 20.0, 40.0, f32::NAN];

    let mean_pyr = MatrixPyramid::new(&src, 2, 2, "nan_test", AggregationOp::Mean, 1);
    let level1 = mean_pyr.levels.last().unwrap();
    assert_eq!(level1.values.len(), 1);
    // Mean of 20.0 and 40.0 (filtering out NaNs) = 30.0
    assert_eq!(level1.values[0], 30.0);

    // All NaNs grid:
    let all_nans = vec![f32::NAN, f32::NAN, f32::NAN, f32::NAN];
    let all_nan_pyr = MatrixPyramid::new(&all_nans, 2, 2, "all_nans", AggregationOp::Mean, 1);
    assert!(all_nan_pyr.levels.last().unwrap().values[0].is_nan());
}

#[test]
fn test_pyramid_level_selection() {
    let width = 4096;
    let height = 4096;
    let values = vec![1.0; width * height];
    let pyramid = MatrixPyramid::new(
        &values,
        width,
        height,
        "selection",
        AggregationOp::Mean,
        512,
    );

    // Zoomed all the way out (span 1.0) into a 1024px canvas -> selects coarser level
    let level_zoomed_out = pyramid.select_level(1.0, 1024);
    assert!(level_zoomed_out >= 2);

    // Zoomed way in (span 0.1) into a 1024px canvas -> selects full/near resolution
    let level_zoomed_in = pyramid.select_level(0.1, 1024);
    assert_eq!(level_zoomed_in, 0);
}

#[test]
fn test_sample_viewport() {
    let width = 1000;
    let height = 1000;
    let mut values = vec![0.0f32; width * height];
    // Put a hotspot in top-left (0..100, 0..100)
    for y in 0..100 {
        for x in 0..100 {
            values[y * width + x] = 100.0;
        }
    }

    let pyramid = MatrixPyramid::new(&values, width, height, "viewport", AggregationOp::Mean, 256);

    // Sample the top-left subregion [0.0..0.1] x [0.0..0.1]
    let sample = pyramid.sample_viewport((0.0, 0.1), (0.0, 0.1), (50, 50));
    assert_eq!(sample.width, 50);
    assert_eq!(sample.height, 50);
    assert_eq!(sample.values[0], 100.0);

    // Sample the bottom-right subregion [0.5..1.0] x [0.5..1.0]
    let sample_br = pyramid.sample_viewport((0.5, 1.0), (0.5, 1.0), (50, 50));
    assert_eq!(sample_br.width, 50);
    assert_eq!(sample_br.height, 50);
    assert_eq!(sample_br.values[0], 0.0);
}

#[test]
fn test_viewport_resampler_visible_bounds() {
    // Identity view: pan [0, 0], zoom 1.0, aspect [1, 1]
    let ((u0, u1), (v0, v1)) =
        ViewportResampler::compute_visible_data_bounds([0.0, 0.0], 1.0, [1.0, 1.0]);
    assert!((u0 - 0.0).abs() < 1e-4);
    assert!((u1 - 1.0).abs() < 1e-4);
    assert!((v0 - 0.0).abs() < 1e-4);
    assert!((v1 - 1.0).abs() < 1e-4);

    // Zoomed 2x: visible region should be centered and span [0.25, 0.75]
    let ((u0_z, u1_z), (v0_z, v1_z)) =
        ViewportResampler::compute_visible_data_bounds([0.0, 0.0], 2.0, [1.0, 1.0]);
    assert!((u0_z - 0.25).abs() < 1e-4);
    assert!((u1_z - 0.75).abs() < 1e-4);
    assert!((v0_z - 0.25).abs() < 1e-4);
    assert!((v1_z - 0.75).abs() < 1e-4);
}

#[test]
fn test_compute_target_resolution_preserves_aspect_ratio() {
    // 50,000 x 20,000 matrix (2.5 aspect ratio)
    let (tw, th) = ViewportResampler::compute_target_resolution(50000, 20000, 2048);
    assert_eq!(tw, 2048);
    assert_eq!(th, 819);
    let ratio = tw as f64 / th as f64;
    assert!((ratio - 2.5).abs() < 0.01);

    // 1,000 x 4,000 matrix (0.25 aspect ratio)
    let (tw2, th2) = ViewportResampler::compute_target_resolution(1000, 4000, 2048);
    assert_eq!(tw2, 512);
    assert_eq!(th2, 2048);
    let ratio2 = tw2 as f64 / th2 as f64;
    assert!((ratio2 - 0.25).abs() < 0.01);
}

#[test]
fn test_resample_if_needed_aspect_ratio_and_lod() {
    let width = 2000;
    let height = 1000;
    let values = vec![5.0f32; width * height];
    let pyramid = std::sync::Arc::new(MatrixPyramid::new(
        &values,
        width,
        height,
        "aspect_test",
        AggregationOp::Mean,
        256,
    ));

    let mut resampler = ViewportResampler::new(Some(pyramid));
    let (tw, th) = ViewportResampler::compute_target_resolution(width, height, 2048);
    assert_eq!(tw, 2000);
    assert_eq!(th, 1000);

    // Initial resample
    let sample1 = resampler.resample_if_needed((0.0, 1.0), (0.0, 1.0), tw, th);
    assert!(sample1.is_some());
    let s1 = sample1.unwrap();
    assert_eq!(s1.data.width, 2000);
    assert_eq!(s1.data.height, 1000);
    assert_eq!(s1.tile_bounds, [0.0, 0.0, 1.0, 1.0]);

    // Identical visible span should skip
    let sample2 = resampler.resample_if_needed((0.0, 1.0), (0.0, 1.0), tw, th);
    assert!(sample2.is_none());

    // Zoom in 10x (visible 0.45..0.55) -> extracts high-res sub-tile with bounds
    let sample3 = resampler.resample_if_needed((0.45, 0.55), (0.45, 0.55), tw, th);
    assert!(sample3.is_some());
    let s3 = sample3.unwrap();
    assert!(s3.tile_bounds[0] <= 0.45 && s3.tile_bounds[2] >= 0.55);
}
