//! Forward and inverse color scale transformations (linear, log, symlog, sqrt, exp).

/// Unscales a normalized colorbar position t in [0.0, 1.0] back to the exact data value v(t) for tick labeling.
pub fn unscale_norm_to_value(
    t: f32,
    cmin: f32,
    cmax: f32,
    scale_type: u32,
    scale_param: f32,
) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let range = (cmax - cmin).max(1e-30);

    match scale_type {
        1 => {
            if cmin < -1e-15 {
                return cmin + t * range;
            }
            let safe_min = if cmin <= 1e-15 {
                (cmax * 0.001).max(1e-12)
            } else {
                cmin
            };
            let safe_max = cmax.max(safe_min * 1.0001);
            let gamma = if scale_param > 0.0 && scale_param != 1.0 {
                scale_param
            } else {
                1.0
            };
            let t_gamma = t.powf(1.0 / gamma);

            let log_min = safe_min.ln();
            let log_max = safe_max.ln();
            let log_val = log_min + t_gamma * (log_max - log_min);
            log_val.exp()
        }
        2 => {
            let c = if scale_param > 0.0 { scale_param } else { 1.0 };
            let safe_range = range.abs().max(1e-6);
            let k = 1.0 + safe_range / c;
            let norm_x = (c * (k.powf(t) - 1.0)) / safe_range;
            cmin + norm_x.clamp(0.0, 1.0) * range
        }
        3 => {
            let y = 2.0 * t - 1.0;
            let norm_x = 0.5 + 0.5 * y.signum() * y.powi(2);
            cmin + norm_x.clamp(0.0, 1.0) * range
        }
        4 => {
            let k = if scale_param > 0.0 { scale_param } else { 3.0 };
            let denom = k.exp() - 1.0;
            let norm_x = if denom.abs() > 1e-5 {
                (1.0 + t * denom).ln() / k
            } else {
                t
            };
            cmin + norm_x.clamp(0.0, 1.0) * range
        }
        _ => cmin + t * range,
    }
}

/// Computes normalized colormap parameter t for raw value val under current scale.
pub fn apply_color_scale_cpu(
    val: f32,
    cmin: f32,
    cmax: f32,
    scale_type: u32,
    scale_param: f32,
) -> f32 {
    let range = (cmax - cmin).max(1e-30);

    match scale_type {
        1 => {
            if cmin < -1e-15 {
                return ((val - cmin) / range).clamp(0.0, 1.0);
            }
            let safe_min = if cmin <= 1e-15 {
                (cmax * 0.001).max(1e-12)
            } else {
                cmin
            };
            let safe_max = cmax.max(safe_min * 1.0001);
            if val <= safe_min {
                return 0.0;
            }
            let safe_v = val.clamp(safe_min, safe_max);
            let log_v = safe_v.ln();
            let log_min = safe_min.ln();
            let log_max = safe_max.ln();
            let log_range = (log_max - log_min).max(1e-6);
            let norm_log = ((log_v - log_min) / log_range).clamp(0.0, 1.0);
            let gamma = if scale_param > 0.0 && scale_param != 1.0 {
                scale_param
            } else {
                1.0
            };
            norm_log.powf(gamma)
        }
        2 => {
            let c = if scale_param > 0.0 { scale_param } else { 1.0 };
            let norm_x = ((val - cmin) / range).clamp(0.0, 1.0);
            let safe_range = range.abs().max(1e-6);
            let num = (c + norm_x * safe_range).ln() - c.ln();
            let denom = (c + safe_range).ln() - c.ln();
            if denom != 0.0 {
                (num / denom).clamp(0.0, 1.0)
            } else {
                norm_x
            }
        }
        3 => {
            let norm_x = ((val - cmin) / range).clamp(0.0, 1.0);
            let x_centered = 2.0 * norm_x - 1.0;
            (0.5 + 0.5 * x_centered.signum() * x_centered.abs().sqrt()).clamp(0.0, 1.0)
        }
        4 => {
            let norm_x = ((val - cmin) / range).clamp(0.0, 1.0);
            let k = if scale_param > 0.0 { scale_param } else { 3.0 };
            let num = (norm_x * k).exp() - 1.0;
            let denom = k.exp() - 1.0;
            if denom.abs() > 1e-5 {
                (num / denom).clamp(0.0, 1.0)
            } else {
                norm_x
            }
        }
        _ => ((val - cmin) / range).clamp(0.0, 1.0),
    }
}
