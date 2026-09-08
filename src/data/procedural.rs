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

/// Generates Clenshaw-Curtis non-linear coordinates spanning [-180, 180] longitude and [90, -90] latitude.
pub fn generate_clenshaw_curtis_coords(nx: usize, ny: usize) -> (Vec<f64>, Vec<f64>) {
    let nx = nx.max(2);
    let ny = ny.max(2);
    let x_coords: Vec<f64> = (0..nx)
        .map(|i| {
            let theta = std::f64::consts::PI * i as f64 / (nx - 1) as f64;
            -180.0 * theta.cos()
        })
        .collect();
    let y_coords: Vec<f64> = (0..ny)
        .map(|j| {
            let theta = std::f64::consts::PI * j as f64 / (ny - 1) as f64;
            90.0 * theta.cos()
        })
        .collect();
    (x_coords, y_coords)
}

/// Generates a synthetic 2D wave harmonic field on a Clenshaw-Curtis grid.
pub fn generate_clenshaw_curtis_2d(nx: usize, ny: usize, timestep: usize) -> (Vec<f32>, f32, f32) {
    let (xs, ys) = generate_clenshaw_curtis_coords(nx, ny);
    let mut data = Vec::with_capacity(nx * ny);
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;
    let t_phase = (timestep % 360) as f64 * 0.05;

    for y in &ys {
        let lat_rad = y.to_radians();
        for x in &xs {
            let lon_rad = x.to_radians();
            let wave = ((2.0 * lon_rad + t_phase).sin() * (3.0 * lat_rad).cos() * 0.5 + 0.5) as f32;
            let val = (wave * 80.0 + 10.0).clamp(0.0, 100.0);
            min_val = min_val.min(val);
            max_val = max_val.max(val);
            data.push(val);
        }
    }
    if min_val > max_val {
        min_val = 0.0;
        max_val = 100.0;
    }
    (data, min_val, max_val)
}

/// Generates Gaussian latitude grid coordinates with regular longitude.
pub fn generate_gaussian_coords(nx: usize, ny: usize) -> (Vec<f64>, Vec<f64>) {
    let nx = nx.max(2);
    let ny = ny.max(2);
    let x_coords: Vec<f64> = (0..nx)
        .map(|i| -180.0 + i as f64 * (360.0 / nx as f64))
        .collect();
    let y_coords: Vec<f64> = (0..ny)
        .map(|j| {
            let mu = (1.0 - (2.0 * j as f64 + 1.0) / ny as f64) * (std::f64::consts::FRAC_PI_2);
            90.0 * mu.sin()
        })
        .collect();
    (x_coords, y_coords)
}

/// Generates a synthetic Rossby/baroclinic wave packet on a Gaussian grid.
pub fn generate_gaussian_grid_2d(nx: usize, ny: usize, timestep: usize) -> (Vec<f32>, f32, f32) {
    let (xs, ys) = generate_gaussian_coords(nx, ny);
    let mut data = Vec::with_capacity(nx * ny);
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;
    let t_phase = (timestep % 360) as f64 * 0.08;

    for y in &ys {
        let lat_rad = y.to_radians();
        let cos_lat = lat_rad.cos() as f32;
        for x in &xs {
            let lon_rad = x.to_radians();
            let wave = (4.0 * lon_rad + t_phase).cos() as f32 * cos_lat.powi(2);
            let val = (50.0 + 40.0 * wave).clamp(0.0, 100.0);
            min_val = min_val.min(val);
            max_val = max_val.max(val);
            data.push(val);
        }
    }
    if min_val > max_val {
        min_val = 0.0;
        max_val = 100.0;
    }
    (data, min_val, max_val)
}

/// Generates geometrically stretched regional coordinates [10°E..50°E] x [30°N..60°N].
pub fn generate_stretched_regional_coords(nx: usize, ny: usize) -> (Vec<f64>, Vec<f64>) {
    let nx = nx.max(2);
    let ny = ny.max(2);
    let x_coords: Vec<f64> = (0..nx)
        .map(|i| {
            let t = i as f64 / (nx - 1) as f64;
            let stretch = (4.0f64.powf(t) - 1.0) / 3.0; // 4x progressive stretch
            10.0 + 40.0 * stretch
        })
        .collect();
    let y_coords: Vec<f64> = (0..ny)
        .map(|j| {
            let t = j as f64 / (ny - 1) as f64;
            let stretch = (3.0f64.powf(t) - 1.0) / 2.0; // 3x progressive stretch
            60.0 - 30.0 * stretch
        })
        .collect();
    (x_coords, y_coords)
}

/// Generates an alternating checkerboard pattern to highlight non-linear cell geometric stretching.
pub fn generate_stretched_regional_2d(nx: usize, ny: usize) -> (Vec<f32>, f32, f32) {
    let mut data = Vec::with_capacity(nx * ny);
    for j in 0..ny {
        for i in 0..nx {
            let val = if (i + j) % 2 == 0 { 85.0 } else { 15.0 };
            data.push(val);
        }
    }
    (data, 15.0, 85.0)
}

/// Generates stepped multi-resolution coordinates with a 5x resolution jump between left and right halves.
pub fn generate_stepped_resolution_coords(nx: usize, ny: usize) -> (Vec<f64>, Vec<f64>) {
    let nx = nx.max(4);
    let ny = ny.max(2);
    let mid_x = nx / 2;
    let mut x_coords = Vec::with_capacity(nx);

    // Left half: [-40, 0] fine resolution
    for i in 0..mid_x {
        let t = i as f64 / mid_x as f64;
        x_coords.push(-40.0 + 40.0 * t);
    }
    // Right half: [0, 40] coarse resolution
    let right_count = nx - mid_x;
    for i in 0..right_count {
        let t = i as f64 / (right_count - 1).max(1) as f64;
        x_coords.push(0.0 + 40.0 * t);
    }

    let y_coords: Vec<f64> = (0..ny)
        .map(|j| {
            let t = j as f64 / (ny - 1) as f64;
            20.0 - 40.0 * t
        })
        .collect();

    (x_coords, y_coords)
}

/// Generates alternating block stripes for the stepped multi-resolution grid.
pub fn generate_stepped_resolution_2d(nx: usize, ny: usize) -> (Vec<f32>, f32, f32) {
    let mut data = Vec::with_capacity(nx * ny);
    for j in 0..ny {
        for i in 0..nx {
            let val = if (i / 2 + j / 2) % 2 == 0 { 90.0 } else { 10.0 };
            data.push(val);
        }
    }
    (data, 10.0, 90.0)
}

/// Generates a synthetic 2D tripolar / ORCA-like deformed curvilinear ocean grid and temperature field.
pub fn generate_curvilinear_orca_grid(
    nx: usize,
    ny: usize,
    timestep: usize,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, f32, f32) {
    let nx = nx.max(4);
    let ny = ny.max(4);
    let total = nx * ny;
    let mut lons = Vec::with_capacity(total);
    let mut lats = Vec::with_capacity(total);
    let mut data = Vec::with_capacity(total);
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;
    let t_phase = (timestep % 360) as f32 * 0.05;

    for j in 0..ny {
        let v = j as f32 / (ny - 1) as f32;
        let base_lat = -80.0 + 170.0 * v; // -80° to +90°

        for i in 0..nx {
            let u = i as f32 / (nx - 1) as f32;
            let base_lon = -180.0 + 360.0 * u; // -180° to +180°

            // Deform northern hemisphere coordinates towards two pseudo-poles
            let (lon, lat) = if base_lat > 20.0 {
                let nh_factor = ((base_lat - 20.0) / 70.0).clamp(0.0, 1.0);
                let lon_warp = (base_lon.to_radians() * 2.0).sin() * 18.0 * nh_factor;
                let lat_warp = (base_lon.to_radians() * 2.0).cos() * 8.0 * nh_factor;
                (
                    (base_lon + lon_warp).clamp(-180.0, 180.0),
                    (base_lat + lat_warp).clamp(-90.0, 90.0),
                )
            } else {
                (base_lon, base_lat)
            };

            // Ocean temperature field with equatorial warm pool, cold poles, and meandering gyre
            let lat_rad = lat.to_radians();
            let lon_rad = lon.to_radians();
            let sst_base = 28.0 * lat_rad.cos().powi(2);
            let meander = (3.0 * lon_rad + t_phase).sin() * (2.0 * lat_rad).cos() * 5.0;
            let val = (sst_base + meander).clamp(-2.0, 35.0);

            min_val = min_val.min(val);
            max_val = max_val.max(val);
            lons.push(lon);
            lats.push(lat);
            data.push(val);
        }
    }

    if min_val > max_val {
        min_val = 0.0;
        max_val = 30.0;
    }

    (lons, lats, data, min_val, max_val)
}

/// Generates a swirling sheared atmospheric curvilinear mesh and spiral wave field.
pub fn generate_curvilinear_swirl_grid(
    nx: usize,
    ny: usize,
    timestep: usize,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, f32, f32) {
    let nx = nx.max(4);
    let ny = ny.max(4);
    let total = nx * ny;
    let mut lons = Vec::with_capacity(total);
    let mut lats = Vec::with_capacity(total);
    let mut data = Vec::with_capacity(total);
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;
    let t_phase = (timestep % 360) as f32 * 0.08;

    for j in 0..ny {
        let v = (j as f32 / (ny - 1) as f32) * 2.0 - 1.0; // [-1, 1]
        for i in 0..nx {
            let u = (i as f32 / (nx - 1) as f32) * 2.0 - 1.0; // [-1, 1]
            let r = (u * u + v * v).sqrt();
            let theta = v.atan2(u);

            // Vortex swirl coordinate shear
            let twist = (-r * 2.5).exp() * 1.8;
            let warped_theta = theta + twist;
            let warped_x = r * warped_theta.cos();
            let warped_y = r * warped_theta.sin();

            let lon = (warped_x * 50.0).clamp(-180.0, 180.0);
            let lat = (warped_y * 35.0 + 20.0).clamp(-90.0, 90.0);

            let spiral =
                ((4.0 * theta - 3.0 * r * std::f32::consts::PI + t_phase).sin() * 0.5 + 0.5) * 80.0;
            let val = spiral.clamp(0.0, 100.0);

            min_val = min_val.min(val);
            max_val = max_val.max(val);
            lons.push(lon);
            lats.push(lat);
            data.push(val);
        }
    }

    if min_val > max_val {
        min_val = 0.0;
        max_val = 100.0;
    }

    (lons, lats, data, min_val, max_val)
}

/// Generates a regional curvilinear grid straddling the antimeridian (150°E to 150°W across 180°).
pub fn generate_curvilinear_antimeridian_grid(
    nx: usize,
    ny: usize,
    timestep: usize,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, f32, f32) {
    let nx = nx.max(4);
    let ny = ny.max(4);
    let total = nx * ny;
    let mut lons = Vec::with_capacity(total);
    let mut lats = Vec::with_capacity(total);
    let mut data = Vec::with_capacity(total);
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;
    let t_phase = (timestep % 360) as f32 * 0.06;

    for j in 0..ny {
        let v = j as f32 / (ny - 1) as f32;
        let base_lat = 10.0 + 60.0 * v; // 10°N to 70°N (Bering Sea / North Pacific)

        for i in 0..nx {
            let u = i as f32 / (nx - 1) as f32;
            // 150°E (150°) to 210°E (-150°W)
            let deg_span = 150.0 + 60.0 * u;
            let lon = if deg_span > 180.0 {
                deg_span - 360.0
            } else {
                deg_span
            };

            // Curvilinear wavy deformation
            let lat_wave = (u * std::f32::consts::PI * 2.0 + t_phase).sin() * 5.0;
            let lat = (base_lat + lat_wave).clamp(-90.0, 90.0);

            let val = (((u * 6.0 + t_phase).sin() * (v * 4.0).cos() * 0.5 + 0.5) * 100.0)
                .clamp(0.0, 100.0);

            min_val = min_val.min(val);
            max_val = max_val.max(val);
            lons.push(lon);
            lats.push(lat);
            data.push(val);
        }
    }

    if min_val > max_val {
        min_val = 0.0;
        max_val = 100.0;
    }

    (lons, lats, data, min_val, max_val)
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
}
