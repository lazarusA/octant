//! Analytical 4D Gaussian wave packet / moving pulse ground-truth mathematical formulas.

use std::collections::HashMap;

use crate::data::octant_block::OctantBlock;

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
#[allow(clippy::too_many_arguments)]
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

    let fx = if nx <= 1 {
        0.5
    } else {
        x as f32 / (nx - 1) as f32
    };
    let fy = if ny <= 1 {
        0.5
    } else {
        y as f32 / (ny - 1) as f32
    };
    let fz = if nz <= 1 {
        0.5
    } else {
        z as f32 / (nz - 1) as f32
    };
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

/// Creates a standard 4D `OctantBlock` wrapping the known-truth scalar field with shape `[nt, nz, ny, nx]`.
pub fn generate_known_truth_4d_block(
    var_name: impl Into<String>,
    nt: usize,
    nz: usize,
    ny: usize,
    nx: usize,
) -> OctantBlock {
    let (values, _, _) = super::fields::generate_procedural_volume_4d(nt, nz, ny, nx);
    let mut coords = HashMap::new();
    coords.insert("lon".to_string(), vec![-180.0, 180.0]);
    coords.insert("lat".to_string(), vec![90.0, -90.0]);
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
        coords,
        HashMap::new(),
    )
}
