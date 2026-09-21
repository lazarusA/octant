//! Synthetic scalar field generators for 2D, 3D, and 4D datasets.

use super::known_truth::{KnownTruth4DParams, eval_known_truth_4d};

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
