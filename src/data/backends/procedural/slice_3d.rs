//! 4D Gaussian wave packet and procedural matrix block slicing.

use std::collections::HashMap;

use crate::data::octant_block::OctantBlock;
use crate::data::procedural::{eval_known_truth_4d, generate_procedural_matrix};
use crate::data::slice_request::SliceRequest;

pub fn slice_procedural_matrix_block(request: &SliceRequest) -> OctantBlock {
    let (h_full, w_full) = (64, 64);
    let (y_start, y_end) = request
        .selections
        .first()
        .map(|s| s.bounds())
        .unwrap_or((0, h_full));
    let (x_start, x_end) = request
        .selections
        .get(1)
        .map(|s| s.bounds())
        .unwrap_or((0, w_full));

    let y_start = y_start.min(h_full);
    let y_end = y_end.min(h_full).max(y_start);
    let x_start = x_start.min(w_full);
    let x_end = x_end.min(w_full).max(x_start);

    let block_h = y_end - y_start;
    let block_w = x_end - x_start;
    let (full_matrix, _, _) = generate_procedural_matrix(w_full, h_full, 0);

    let mut values = Vec::with_capacity(block_h * block_w);
    for y in y_start..y_end {
        for x in x_start..x_end {
            let idx = y * w_full + x;
            values.push(full_matrix.get(idx).copied().unwrap_or(0.0));
        }
    }

    let mut coords = HashMap::new();
    let t_start_x = if w_full > 1 {
        x_start as f64 / (w_full - 1) as f64
    } else {
        0.0
    };
    let t_end_x = if w_full > 1 {
        (x_end.saturating_sub(1)) as f64 / (w_full - 1) as f64
    } else {
        1.0
    };
    coords.insert(
        "x".to_string(),
        vec![-180.0 + t_start_x * 360.0, -180.0 + t_end_x * 360.0],
    );

    let t_start_y = if h_full > 1 {
        y_start as f64 / (h_full - 1) as f64
    } else {
        0.0
    };
    let t_end_y = if h_full > 1 {
        (y_end.saturating_sub(1)) as f64 / (h_full - 1) as f64
    } else {
        1.0
    };
    coords.insert(
        "y".to_string(),
        vec![90.0 - t_start_y * 180.0, 90.0 - t_end_y * 180.0],
    );

    OctantBlock::new(
        request.variable.clone(),
        vec![block_h, block_w],
        vec!["y".to_string(), "x".to_string()],
        vec![y_start, x_start],
        values,
        coords,
        HashMap::new(),
    )
}

pub fn slice_gaussian_wave_packet_4d(request: &SliceRequest) -> OctantBlock {
    let (nt_full, nz_full, ny_full, nx_full) = (20, 32, 32, 32);

    let (t_start, t_end) = request
        .selections
        .first()
        .map(|s| s.bounds())
        .unwrap_or((0, nt_full));
    let (z_start, z_end) = request
        .selections
        .get(1)
        .map(|s| s.bounds())
        .unwrap_or((0, nz_full));
    let (y_start, y_end) = request
        .selections
        .get(2)
        .map(|s| s.bounds())
        .unwrap_or((0, ny_full));
    let (x_start, x_end) = request
        .selections
        .get(3)
        .map(|s| s.bounds())
        .unwrap_or((0, nx_full));

    let t_start = t_start.min(nt_full);
    let t_end = t_end.min(nt_full).max(t_start);
    let z_start = z_start.min(nz_full);
    let z_end = z_end.min(nz_full).max(z_start);
    let y_start = y_start.min(ny_full);
    let y_end = y_end.min(ny_full).max(y_start);
    let x_start = x_start.min(nx_full);
    let x_end = x_end.min(nx_full).max(x_start);

    let dt = (t_end - t_start).max(1);
    let dz = (z_end - z_start).max(1);
    let dy = (y_end - y_start).max(1);
    let dx = (x_end - x_start).max(1);

    let total = dt * dz * dy * dx;
    let mut values = Vec::with_capacity(total);

    for t in t_start..t_end {
        for z in z_start..z_end {
            for y in y_start..y_end {
                for x in x_start..x_end {
                    let val =
                        eval_known_truth_4d(t, nt_full, z, nz_full, y, ny_full, x, nx_full, None);
                    values.push(val);
                }
            }
        }
    }

    let mut coords = HashMap::new();
    let t_start_lon = if nx_full > 1 {
        x_start as f64 / (nx_full - 1) as f64
    } else {
        0.0
    };
    let t_end_lon = if nx_full > 1 {
        (x_end.saturating_sub(1)) as f64 / (nx_full - 1) as f64
    } else {
        1.0
    };
    coords.insert(
        "lon".to_string(),
        vec![-180.0 + t_start_lon * 360.0, -180.0 + t_end_lon * 360.0],
    );

    let t_start_lat = if ny_full > 1 {
        y_start as f64 / (ny_full - 1) as f64
    } else {
        0.0
    };
    let t_end_lat = if ny_full > 1 {
        (y_end.saturating_sub(1)) as f64 / (ny_full - 1) as f64
    } else {
        1.0
    };
    coords.insert(
        "lat".to_string(),
        vec![90.0 - t_start_lat * 180.0, 90.0 - t_end_lat * 180.0],
    );

    let z_start_m = if nz_full > 1 {
        z_start as f64 * (1000.0 / (nz_full - 1) as f64)
    } else {
        0.0
    };
    let z_end_m = if nz_full > 1 {
        (z_end.saturating_sub(1)) as f64 * (1000.0 / (nz_full - 1) as f64)
    } else {
        1000.0
    };
    coords.insert("depth".to_string(), vec![z_start_m, z_end_m]);
    coords.insert(
        "time".to_string(),
        vec![t_start as f64, (t_end.saturating_sub(1)) as f64],
    );

    OctantBlock::new(
        request.variable.clone(),
        vec![
            t_end - t_start,
            z_end - z_start,
            y_end - y_start,
            x_end - x_start,
        ],
        vec![
            "time".to_string(),
            "depth".to_string(),
            "lat".to_string(),
            "lon".to_string(),
        ],
        vec![t_start, z_start, y_start, x_start],
        values,
        coords,
        HashMap::new(),
    )
}
