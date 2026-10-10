//! Colorbar tick generation and scientific notation formatting.

#[derive(Clone, Copy)]
pub struct ColorbarTick {
    pub t_pos: f32,
    pub val: f32,
    pub is_major: bool,
    /// Whether the tick shows its value (`ScientificTick(val)`), formatted
    /// when drawn.
    pub labeled: bool,
}

impl ColorbarTick {
    const EMPTY: Self = Self {
        t_pos: 0.0,
        val: 0.0,
        is_major: false,
        labeled: false,
    };
}

/// Decades of `f32`, from its smallest subnormal to its largest value: log
/// ranges are clamped to them, so an infinite bound still ends the loop.
const F32_DECADES: (i32, i32) = (-46, 39);
/// Widest log range (in decades) that gets minor ticks; past it they would
/// sit a pixel or two apart.
const MINOR_DECADES: i32 = 16;
/// Room for every tick: a log range with minor ticks (its decades plus one on
/// either side, nine ticks each), or one major tick per `f32` decade, plus
/// both ends.
const MAX_TICKS: usize = {
    let with_minor = (MINOR_DECADES as usize + 2) * 9 + 2;
    let majors = (F32_DECADES.1 - F32_DECADES.0) as usize + 2 + 2;
    if with_minor > majors {
        with_minor
    } else {
        majors
    }
};

/// A bar's ticks in a stack array, so drawing them allocates nothing.
pub struct ColorbarTicks {
    ticks: [ColorbarTick; MAX_TICKS],
    len: usize,
}

impl ColorbarTicks {
    fn new() -> Self {
        Self {
            ticks: [ColorbarTick::EMPTY; MAX_TICKS],
            len: 0,
        }
    }

    fn push(&mut self, tick: ColorbarTick) {
        if let Some(slot) = self.ticks.get_mut(self.len) {
            *slot = tick;
            self.len += 1;
        }
    }

    fn sort(&mut self) {
        self.ticks[..self.len].sort_unstable_by(|a, b| a.t_pos.total_cmp(&b.t_pos));
    }
}

impl std::ops::Deref for ColorbarTicks {
    type Target = [ColorbarTick];

    fn deref(&self) -> &[ColorbarTick] {
        &self.ticks[..self.len]
    }
}

impl std::ops::DerefMut for ColorbarTicks {
    fn deref_mut(&mut self) -> &mut [ColorbarTick] {
        &mut self.ticks[..self.len]
    }
}

/// Generates linear or logarithmic ticks and subdivisions across [min_val, max_val].
pub fn generate_colorbar_ticks(
    min_val: f32,
    max_val: f32,
    scale_type: u32,
    scale_param: f32,
) -> ColorbarTicks {
    let mut ticks = ColorbarTicks::new();

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
            let (lo, hi) = F32_DECADES;
            let dec_start = (log_min.floor() as i32).clamp(lo, hi);
            let dec_end = (log_max.ceil() as i32).clamp(lo, hi);
            let minor = dec_end - dec_start <= MINOR_DECADES;

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
                        labeled: true,
                    });
                }

                // Minor ticks at m * 10^dec for m in 2..9
                for m in (2..10).filter(|_| minor) {
                    let m_val = m as f32 * base;
                    if m_val > safe_min && m_val < safe_max {
                        let norm_linear = ((m_val.log10() - log_min) / log_range).clamp(0.0, 1.0);
                        let t_pos = norm_linear.powf(gamma);
                        ticks.push(ColorbarTick {
                            t_pos,
                            val: m_val,
                            is_major: false,
                            labeled: false,
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
                    labeled: true,
                });
            }
            if !ticks.iter().any(|t| (t.t_pos - 1.0).abs() < 0.02) {
                ticks.push(ColorbarTick {
                    t_pos: 1.0,
                    val: safe_max,
                    is_major: true,
                    labeled: true,
                });
            }

            ticks.sort();
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
            labeled: true,
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
                labeled: false,
            });
        }
    }

    ticks.sort();

    // De-cluttering collision pass
    let min_label_spacing = 0.08;
    let mut last_labeled_t: Option<f32> = None;

    for tick in ticks.iter_mut() {
        if tick.is_major && tick.labeled {
            if let Some(last_t) = last_labeled_t {
                if (tick.t_pos - last_t).abs() < min_label_spacing
                    && (1.0 - tick.t_pos).abs() > 0.01
                {
                    tick.labeled = false;
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

pub use crate::utils::math::{ScientificTick, TICK_BUF, format_scientific_tick};
