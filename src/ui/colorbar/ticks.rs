//! Colorbar tick generation and scientific notation formatting.

pub struct ColorbarTick {
    pub t_pos: f32,
    pub val: f32,
    pub is_major: bool,
    pub label: Option<String>,
}

/// Generates linear or logarithmic ticks and subdivisions across [min_val, max_val].
pub fn generate_colorbar_ticks(
    min_val: f32,
    max_val: f32,
    scale_type: u32,
    scale_param: f32,
) -> Vec<ColorbarTick> {
    let mut ticks = Vec::new();

    if scale_type == 1 {
        // Logarithmic scale major (powers of 10) & minor (2..9 subdivisions per decade)
        let safe_min = if min_val <= 1e-15 {
            1e-12_f32.min(max_val * 1e-6)
        } else {
            min_val
        };
        let safe_max = max_val.max(safe_min * 1.0001);

        let log_min = safe_min.log10();
        let log_max = safe_max.log10();
        let log_range = (log_max - log_min).max(1e-6);
        let gamma = if scale_param > 0.0 && scale_param != 1.0 {
            scale_param
        } else {
            1.0
        };

        if log_range >= 0.8 {
            let dec_start = log_min.floor() as i32;
            let dec_end = log_max.ceil() as i32;

            for dec in (dec_start - 1)..=dec_end {
                let base = 10.0_f32.powi(dec);

                // Major tick at 1 * 10^dec
                if base >= safe_min * 0.999 && base <= safe_max * 1.001 {
                    let norm_linear = ((base.log10() - log_min) / log_range).clamp(0.0, 1.0);
                    let t_pos = norm_linear.powf(gamma);
                    ticks.push(ColorbarTick {
                        t_pos,
                        val: base,
                        is_major: true,
                        label: Some(format_scientific_tick(base)),
                    });
                }

                // Minor ticks at m * 10^dec for m in 2..9
                for m in 2..10 {
                    let m_val = m as f32 * base;
                    if m_val > safe_min && m_val < safe_max {
                        let norm_linear = ((m_val.log10() - log_min) / log_range).clamp(0.0, 1.0);
                        let t_pos = norm_linear.powf(gamma);
                        ticks.push(ColorbarTick {
                            t_pos,
                            val: m_val,
                            is_major: false,
                            label: None,
                        });
                    }
                }
            }

            // Ensure min and max bounds are present as major ticks if not already added
            if !ticks.iter().any(|t| (t.t_pos - 0.0).abs() < 0.02) {
                ticks.push(ColorbarTick {
                    t_pos: 0.0,
                    val: safe_min,
                    is_major: true,
                    label: Some(format_scientific_tick(safe_min)),
                });
            }
            if !ticks.iter().any(|t| (t.t_pos - 1.0).abs() < 0.02) {
                ticks.push(ColorbarTick {
                    t_pos: 1.0,
                    val: safe_max,
                    is_major: true,
                    label: Some(format_scientific_tick(safe_max)),
                });
            }

            ticks.sort_by(|a, b| a.t_pos.total_cmp(&b.t_pos));
            return ticks;
        }
    }

    // Default Linear & Standard Scale Ticks (5 major ticks, 4 minor subdivisions per interval)
    let major_positions = [0.00, 0.25, 0.50, 0.75, 1.00];
    for &t_maj in &major_positions {
        let val = crate::utils::colormap::unscale_norm_to_value(
            t_maj,
            min_val,
            max_val,
            scale_type,
            scale_param,
        );
        ticks.push(ColorbarTick {
            t_pos: t_maj,
            val,
            is_major: true,
            label: Some(format_scientific_tick(val)),
        });
    }

    for i in 0..4 {
        let t_start = major_positions[i];
        let t_end = major_positions[i + 1];
        for step in 1..5 {
            let t_min = t_start + (t_end - t_start) * (step as f32 / 5.0);
            let val = crate::utils::colormap::unscale_norm_to_value(
                t_min,
                min_val,
                max_val,
                scale_type,
                scale_param,
            );
            ticks.push(ColorbarTick {
                t_pos: t_min,
                val,
                is_major: false,
                label: None,
            });
        }
    }

    ticks.sort_by(|a, b| a.t_pos.total_cmp(&b.t_pos));

    // De-cluttering collision pass
    let min_label_spacing = 0.08;
    let mut last_labeled_t: Option<f32> = None;

    for tick in ticks.iter_mut() {
        if tick.is_major && tick.label.is_some() {
            if let Some(last_t) = last_labeled_t {
                if (tick.t_pos - last_t).abs() < min_label_spacing
                    && (1.0 - tick.t_pos).abs() > 0.01
                {
                    tick.label = None;
                } else {
                    last_labeled_t = Some(tick.t_pos);
                }
            } else {
                last_labeled_t = Some(tick.t_pos);
            }
        }
    }

    ticks
}

/// Formats tick values cleanly using integer/decimal or concise scientific notation.
pub fn format_scientific_tick(val: f32) -> String {
    let abs_val = val.abs();
    if abs_val == 0.0 {
        "0".to_string()
    } else if !(0.001..10000.0).contains(&abs_val) {
        let s = format!("{:.2e}", val);
        if let Some((mantissa, exponent)) = s.split_once('e') {
            let clean_mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
            format!("{}e{}", clean_mantissa, exponent)
        } else {
            s
        }
    } else if (val.fract()).abs() < 1e-5 {
        format!("{:.0}", val)
    } else if (val * 10.0).fract().abs() < 1e-5 {
        format!("{:.1}", val)
    } else {
        format!("{:.2}", val)
    }
}
