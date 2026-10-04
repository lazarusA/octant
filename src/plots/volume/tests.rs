//! CPU tests: plane encoding and the uniform layout.

use super::encode::{Dims, encode_rgba, encode_scalar, is_valid};
use super::pipeline::SHADER_HARDWARE_FILTER;
use super::uniforms::VolumeUniforms;

const CUBE: Dims = Dims { w: 3, h: 3, d: 3 };

#[test]
fn missing_values_match_shader_test() {
    assert!(is_valid(0.0) && is_valid(-1e30) && is_valid(1e-38));
    assert!(!is_valid(f32::NAN) && !is_valid(f32::INFINITY) && !is_valid(1.5e30));
}

#[test]
fn valid_voxels_pass_through() {
    let values: Vec<f32> = (0..27).map(|i| i as f32 * 1e-9).collect();
    let out = encode_scalar(&values, CUBE, 0..3);
    assert_eq!(out.texels, values, "small values must survive exactly");
    assert!(out.validity.iter().all(|&v| v == 255));
    assert_eq!(out.invalid_per_plane, vec![0, 0, 0]);
}

#[test]
fn missing_voxel_takes_neighbor_average() {
    let mut values = vec![2.0f32; 27];
    values[13] = f32::NAN; // center
    values[0] = 5.0;
    let out = encode_scalar(&values, CUBE, 0..3);
    let expected = (25.0 * 2.0 + 5.0) / 26.0;
    assert!((out.texels[13] - expected).abs() < 1e-6);
    assert_eq!(out.validity[13], 0);
    assert_eq!(out.invalid_per_plane, vec![0, 1, 0]);
}

#[test]
fn volume_without_data_fills_zeros() {
    let values = vec![f32::NAN; 27];
    let out = encode_scalar(&values, CUBE, 0..3);
    assert!(out.texels.iter().all(|&v| v == 0.0));
    assert!(out.validity.iter().all(|&v| v == 0));
    assert_eq!(out.invalid_per_plane, vec![9, 9, 9]);
}

#[test]
fn plane_range_is_clamped_to_depth() {
    let values = vec![1.0f32; 27];
    let out = encode_scalar(&values, CUBE, 2..9);
    assert_eq!(out.z, 2..3);
    assert_eq!(out.texels.len(), 9);
    assert!(encode_scalar(&values, CUBE, 5..9).texels.is_empty());
    // A short buffer encodes nothing instead of reading out of bounds.
    assert!(encode_scalar(&values[..20], CUBE, 0..3).texels.is_empty());
}

#[test]
fn packed_rgb_unpacks_and_fills() {
    let packed = (10u32 | (20 << 8) | (30 << 16)) as f32;
    let mut values = vec![packed; 27];
    values[13] = f32::NAN;
    let out = encode_rgba(&values, CUBE, 0..3);
    assert_eq!(out.texels[0], [10, 20, 30, 30]);
    assert_eq!(out.texels[13], [10, 20, 30, 30]);
    assert_eq!(out.validity[13], 0);
}

#[test]
fn uniform_layout_matches_wgsl() {
    use wgpu::naga;
    let module =
        naga::front::wgsl::parse_str(SHADER_HARDWARE_FILTER).expect("volume shader parses");
    let mut layouter = naga::proc::Layouter::default();
    layouter.update(module.to_ctx()).expect("layout");
    let (handle, _) = module
        .types
        .iter()
        .find(|(_, ty)| ty.name.as_deref() == Some("Uniforms"))
        .expect("Uniforms struct");
    assert_eq!(
        layouter[handle].size as usize,
        std::mem::size_of::<VolumeUniforms>()
    );
}
