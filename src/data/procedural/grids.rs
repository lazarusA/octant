//! Non-linear coordinate systems and test grid generators.

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
