use super::{
    fields::{
        generate_procedural_matrix, generate_procedural_volume_3d, generate_procedural_volume_4d,
    },
    grids::{
        generate_clenshaw_curtis_2d, generate_clenshaw_curtis_coords, generate_gaussian_coords,
        generate_gaussian_grid_2d, generate_stepped_resolution_2d,
        generate_stepped_resolution_coords, generate_stretched_regional_2d,
        generate_stretched_regional_coords,
    },
    known_truth::{KnownTruth4DParams, eval_known_truth_4d, get_known_truth_4d_center},
};

#[test]
fn test_generate_procedural_matrix_bounds() {
    let (data, min_v, max_v) = generate_procedural_matrix(32, 32, 0);
    assert_eq!(data.len(), 32 * 32);
    assert!(min_v >= 0.0);
    assert!(max_v <= 100.0);
}

#[test]
fn test_eval_known_truth_4d_peak_and_decay() {
    let params = KnownTruth4DParams::default();
    let (x0, y0, z0) = get_known_truth_4d_center(0, 10, Some(&params));

    // Evaluate at the continuous peak position
    let (nx, ny, nz) = (101, 101, 101);
    let center_x = (x0 * (nx - 1) as f32).round() as usize;
    let center_y = (y0 * (ny - 1) as f32).round() as usize;
    let center_z = (z0 * (nz - 1) as f32).round() as usize;

    let peak_val = eval_known_truth_4d(
        0,
        10,
        center_z,
        nz,
        center_y,
        ny,
        center_x,
        nx,
        Some(&params),
    );
    assert!(
        peak_val > 40.0,
        "Expected peak near center to be high, got {peak_val}"
    );

    // Far away from peak (at corner 0,0,0 if peak is around 0.8,0.5,0.2)
    let far_val = eval_known_truth_4d(0, 10, 0, nz, 0, ny, 0, nx, Some(&params));
    assert!(
        far_val < peak_val,
        "Far value {far_val} should be less than peak {peak_val}"
    );
}

#[test]
fn test_generate_procedural_volume_3d_consistency() {
    let (data, min_v, max_v) = generate_procedural_volume_3d(16, 16, 8, 0, 5);
    assert_eq!(data.len(), 16 * 16 * 8);
    assert!(min_v >= 0.0);
    assert!(max_v > min_v);

    // Verify index matches eval_known_truth_4d
    let sample = data[0]; // z=0, y=0, x=0
    let expected = eval_known_truth_4d(0, 5, 0, 8, 0, 16, 0, 16, None);
    assert!((sample - expected).abs() < 1e-5);
}

#[test]
fn test_generate_procedural_volume_4d_consistency() {
    let (data, min_v, max_v) = generate_procedural_volume_4d(4, 8, 8, 8);
    assert_eq!(data.len(), 4 * 8 * 8 * 8);
    assert!(min_v >= 0.0);
    assert!(max_v > min_v);

    // Verify t=2, z=3, y=4, x=5
    let idx = 2 * (8 * 8 * 8) + 3 * (8 * 8) + 4 * 8 + 5;
    let sample = data[idx];
    let expected = eval_known_truth_4d(2, 4, 3, 8, 4, 8, 5, 8, None);
    assert!((sample - expected).abs() < 1e-5);
}

#[test]
fn test_generate_clenshaw_curtis_bounds_and_irregularity() {
    let (xs, ys) = generate_clenshaw_curtis_coords(32, 16);
    assert_eq!(xs.len(), 32);
    assert_eq!(ys.len(), 16);
    assert!((xs[0] - (-180.0)).abs() < 1e-4);
    assert!((xs[31] - 180.0).abs() < 1e-4);
    assert!((ys[0] - 90.0).abs() < 1e-4);
    assert!((ys[15] - (-90.0)).abs() < 1e-4);

    let (data, min_v, max_v) = generate_clenshaw_curtis_2d(32, 16, 0);
    assert_eq!(data.len(), 32 * 16);
    assert!(min_v >= 0.0);
    assert!(max_v <= 100.0);
}

#[test]
fn test_generate_gaussian_grid_bounds_and_values() {
    let (xs, ys) = generate_gaussian_coords(32, 16);
    assert_eq!(xs.len(), 32);
    assert_eq!(ys.len(), 16);
    assert!((xs[0] - (-180.0)).abs() < 1e-4);
    assert!(ys[0] > 0.0 && ys[15] < 0.0);

    let (data, min_v, max_v) = generate_gaussian_grid_2d(32, 16, 0);
    assert_eq!(data.len(), 32 * 16);
    assert!(min_v >= 0.0 && max_v <= 100.0);
}

#[test]
fn test_generate_stretched_regional_bounds_and_checkerboard() {
    let (xs, ys) = generate_stretched_regional_coords(20, 15);
    assert_eq!(xs.len(), 20);
    assert_eq!(ys.len(), 15);
    assert!((xs[0] - 10.0).abs() < 1e-4);
    assert!((xs[19] - 50.0).abs() < 1e-4);
    assert!((ys[0] - 60.0).abs() < 1e-4);
    assert!((ys[14] - 30.0).abs() < 1e-4);

    let (data, min_v, max_v) = generate_stretched_regional_2d(20, 15);
    assert_eq!(data.len(), 20 * 15);
    assert_eq!(min_v, 15.0);
    assert_eq!(max_v, 85.0);
}

#[test]
fn test_generate_stepped_resolution_bounds_and_step() {
    let (xs, ys) = generate_stepped_resolution_coords(32, 16);
    assert_eq!(xs.len(), 32);
    assert_eq!(ys.len(), 16);
    assert!((xs[0] - (-40.0)).abs() < 1e-4);
    assert!((xs[31] - 40.0).abs() < 1e-4);

    let (data, min_v, max_v) = generate_stepped_resolution_2d(32, 16);
    assert_eq!(data.len(), 32 * 16);
    assert_eq!(min_v, 10.0);
    assert_eq!(max_v, 90.0);
}
