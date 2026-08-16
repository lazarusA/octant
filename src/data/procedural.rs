use std::collections::HashMap;

use super::octant_block::OctantBlock;

/// Parameters defining the analytical 4D Gaussian wave packet / moving pulse.
#[derive(Debug, Clone)]
pub struct KnownTruth4DParams {
    /// Standard deviation of Gaussian pulse in normalized [0, 1] spatial coordinates.
    pub sigma: f32,
    /// Base amplitude of the pulse.
    pub base_amplitude: f32,
    /// Amplitude oscillation magnitude across time.
    pub amplitude_modulation: f32,
    /// Background baseline value.
    pub background: f32,
    /// Angular frequency multiplier for orbital trajectory.
    pub orbit_frequency: f32,
}

impl Default for KnownTruth4DParams {
    fn default() -> Self {
        Self {
            sigma: 0.15,
            base_amplitude: 50.0,
            amplitude_modulation: 30.0,
            background: 5.0,
            orbit_frequency: 1.0,
        }
    }
}

/// Computes the exact analytical ground truth center coordinates (x0, y0, z0) in normalized [0, 1]^3 space at timestep `t`.
pub fn get_known_truth_4d_center(
    t: usize,
    num_timesteps: usize,
    params: Option<&KnownTruth4DParams>,
) -> (f32, f32, f32) {
    let default_params = KnownTruth4DParams::default();
    let p = params.unwrap_or(&default_params);

    let ft = if num_timesteps <= 1 {
        0.0
    } else {
        (t % num_timesteps) as f32 / (num_timesteps - 1) as f32
    };

    let phi = std::f32::consts::TAU * p.orbit_frequency * ft;
    let x0 = 0.5 + 0.3 * phi.cos();
    let y0 = 0.5 + 0.3 * phi.sin();
    let z0 = 0.2 + 0.6 * ft;

    (x0, y0, z0)
}

/// Evaluates the analytical ground-truth 4D scalar field at continuous/discrete indices (t, z, y, x).
pub fn eval_known_truth_4d(
    t: usize,
    num_timesteps: usize,
    z: usize,
    nz: usize,
    y: usize,
    ny: usize,
    x: usize,
    nx: usize,
    params: Option<&KnownTruth4DParams>,
) -> f32 {
    let default_params = KnownTruth4DParams::default();
    let p = params.unwrap_or(&default_params);

    let fx = if nx <= 1 { 0.5 } else { x as f32 / (nx - 1) as f32 };
    let fy = if ny <= 1 { 0.5 } else { y as f32 / (ny - 1) as f32 };
    let fz = if nz <= 1 { 0.5 } else { z as f32 / (nz - 1) as f32 };
    let ft = if num_timesteps <= 1 {
        0.0
    } else {
        (t % num_timesteps) as f32 / (num_timesteps - 1) as f32
    };

    let phi = std::f32::consts::TAU * p.orbit_frequency * ft;
    let (x0, y0, z0) = get_known_truth_4d_center(t, num_timesteps, Some(p));
    let amplitude = p.base_amplitude + p.amplitude_modulation * phi.sin();

    let dx = fx - x0;
    let dy = fy - y0;
    let dz = fz - z0;
    let dist_sq = dx * dx + dy * dy + dz * dz;

    let two_sigma_sq = 2.0 * p.sigma * p.sigma;
    let gaussian = (-dist_sq / two_sigma_sq).exp();

    p.background + amplitude * gaussian
}

/// Generates a synthetic 2D scalar field matrix for testing and fallback rendering.
pub fn generate_procedural_matrix(
    width: usize,
    height: usize,
    timestep: usize,
) -> (Vec<f32>, f32, f32) {
    let mut raw_data = Vec::with_capacity(width * height);
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;

    for y in 0..height {
        for x in 0..width {
            let fx = x as f32 / width as f32;
            let fy = y as f32 / height as f32;
            let t_shift = (timestep % 365) as f32 * 0.05;
            let wave1 = ((fx * 8.0 + t_shift).sin() * (fy * 8.0).cos() * 0.5 + 0.5) * 80.0;
            let wave2 = (((x * 23 + y * 47) % 100) as f32) * 0.2;
            let val = (wave1 + wave2).clamp(0.0, 100.0);

            if val < min_val {
                min_val = val;
            }
            if val > max_val {
                max_val = val;
            }

            raw_data.push(val);
        }
    }

    if min_val > max_val {
        min_val = 0.0;
        max_val = 100.0;
    }

    (raw_data, min_val, max_val)
}

/// Generates a synthetic 3D scalar volume slice at timestep `timestep` of `num_timesteps`.
/// Dimensions correspond to `(width: nx, height: ny, depth: nz)`.
/// Data is ordered in standard Z -> Y -> X row-major storage.
pub fn generate_procedural_volume_3d(
    nx: usize,
    ny: usize,
    nz: usize,
    timestep: usize,
    num_timesteps: usize,
) -> (Vec<f32>, f32, f32) {
    let total = nx * ny * nz;
    let mut raw_data = Vec::with_capacity(total);
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;

    let params = KnownTruth4DParams::default();

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let val = eval_known_truth_4d(
                    timestep,
                    num_timesteps,
                    z,
                    nz,
                    y,
                    ny,
                    x,
                    nx,
                    Some(&params),
                );

                if val < min_val {
                    min_val = val;
                }
                if val > max_val {
                    max_val = val;
                }

                raw_data.push(val);
            }
        }
    }

    if min_val > max_val {
        min_val = 0.0;
        max_val = 100.0;
    }

    (raw_data, min_val, max_val)
}

/// Generates a full 4D scalar volume dataset with shape `[nt, nz, ny, nx]`.
/// Layout is row-major `[T, Z, Y, X]`.
pub fn generate_procedural_volume_4d(
    nt: usize,
    nz: usize,
    ny: usize,
    nx: usize,
) -> (Vec<f32>, f32, f32) {
    let total = nt * nz * ny * nx;
    let mut raw_data = Vec::with_capacity(total);
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;

    let params = KnownTruth4DParams::default();

    for t in 0..nt {
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let val = eval_known_truth_4d(t, nt, z, nz, y, ny, x, nx, Some(&params));

                    if val < min_val {
                        min_val = val;
                    }
                    if val > max_val {
                        max_val = val;
                    }

                    raw_data.push(val);
                }
            }
        }
    }

    if min_val > max_val {
        min_val = 0.0;
        max_val = 100.0;
    }

    (raw_data, min_val, max_val)
}

/// Creates a standard 4D `OctantBlock` wrapping the known-truth scalar field with shape `[nt, nz, ny, nx]`.
pub fn generate_known_truth_4d_block(
    var_name: impl Into<String>,
    nt: usize,
    nz: usize,
    ny: usize,
    nx: usize,
) -> OctantBlock {
    let (values, _, _) = generate_procedural_volume_4d(nt, nz, ny, nx);
    OctantBlock::new(
        var_name.into(),
        vec![nt, nz, ny, nx],
        vec![
            "time".to_string(),
            "depth".to_string(),
            "lat".to_string(),
            "lon".to_string(),
        ],
        vec![0, 0, 0, 0],
        values,
        HashMap::new(),
        HashMap::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let peak_val = eval_known_truth_4d(0, 10, center_z, nz, center_y, ny, center_x, nx, Some(&params));
        assert!(peak_val > 40.0, "Expected peak near center to be high, got {peak_val}");

        // Far away from peak (at corner 0,0,0 if peak is around 0.8,0.5,0.2)
        let far_val = eval_known_truth_4d(0, 10, 0, nz, 0, ny, 0, nx, Some(&params));
        assert!(far_val < peak_val, "Far value {far_val} should be less than peak {peak_val}");
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
}
