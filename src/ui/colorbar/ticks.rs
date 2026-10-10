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
    const EMPTY: Self = Self::minor(0.0, 0.0);

    /// A labeled major tick of `val` at `t_pos`.
    const fn major(t_pos: f32, val: f32) -> Self {
        Self {
            t_pos,
            val,
            is_major: true,
            labeled: true,
        }
    }

    /// An unlabeled minor tick of `val` at `t_pos`.
    const fn minor(t_pos: f32, val: f32) -> Self {
        Self {
            t_pos,
            val,
            is_major: false,
            labeled: false,
        }
    }
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
    if scale_type == 1
        && let Some(ticks) = log_ticks(min_val, max_val, scale_param)
    {
        return ticks;
    }
    let mut ticks = linear_ticks(min_val, max_val, scale_type, scale_param);
    declutter(&mut ticks);
    ticks
}

/// Majors at the powers of ten and minors at their 2..9 multiples (when at
/// most `MINOR_DECADES` decades), plus both ends; `None` when the range is
/// under 0.8 decades, which gets linear ticks instead.
fn log_ticks(min_val: f32, max_val: f32, scale_param: f32) -> Option<ColorbarTicks> {
    let safe_min = if min_val <= 1e-15 {
        1e-12_f32.min(max_val * 1e-6)
    } else {
        min_val
    };
    let safe_max = max_val.max(safe_min * 1.0001);
    let (log_min, log_max) = (safe_min.log10(), safe_max.log10());
    let log_range = (log_max - log_min).max(1e-6);
    if log_range < 0.8 {
        return None;
    }
    let gamma = if scale_param > 0.0 && scale_param != 1.0 {
        scale_param
    } else {
        1.0
    };
    let t_of = |val: f32| {
        ((val.log10() - log_min) / log_range)
            .clamp(0.0, 1.0)
            .powf(gamma)
    };
    let (lo, hi) = F32_DECADES;
    let dec_start = (log_min.floor() as i32).clamp(lo, hi);
    let dec_end = (log_max.ceil() as i32).clamp(lo, hi);
    let minor = dec_end - dec_start <= MINOR_DECADES;

    let mut ticks = ColorbarTicks::new();
    for dec in (dec_start - 1)..=dec_end {
        let base = 10.0_f32.powi(dec);
        if base >= safe_min * 0.999 && base <= safe_max * 1.001 {
            ticks.push(ColorbarTick::major(t_of(base), base));
        }
        for m in (2..10).filter(|_| minor) {
            let val = m as f32 * base;
            if val > safe_min && val < safe_max {
                ticks.push(ColorbarTick::minor(t_of(val), val));
            }
        }
    }
    // Both ends get a major tick unless one is already there.
    for (t_pos, val) in [(0.0, safe_min), (1.0, safe_max)] {
        if !ticks.iter().any(|t| (t.t_pos - t_pos).abs() < 0.02) {
            ticks.push(ColorbarTick::major(t_pos, val));
        }
    }
    ticks.sort();
    Some(ticks)
}

/// Five majors at quarters of the bar, with four minors between each two.
fn linear_ticks(min_val: f32, max_val: f32, scale_type: u32, scale_param: f32) -> ColorbarTicks {
    let value_at = |t: f32| {
        crate::utils::colormap::unscale_norm_to_value(t, min_val, max_val, scale_type, scale_param)
    };
    let mut ticks = ColorbarTicks::new();
    for i in 0..=20 {
        let t_pos = i as f32 / 20.0;
        let tick = if i % 5 == 0 {
            ColorbarTick::major(t_pos, value_at(t_pos))
        } else {
            ColorbarTick::minor(t_pos, value_at(t_pos))
        };
        ticks.push(tick);
    }
    ticks
}

/// Hides the label of each major tick too close to the last labeled one,
/// except at the top end.
fn declutter(ticks: &mut ColorbarTicks) {
    const MIN_LABEL_SPACING: f32 = 0.08;
    let mut last_labeled: Option<f32> = None;
    for tick in ticks.iter_mut().filter(|t| t.is_major && t.labeled) {
        let crowded = last_labeled.is_some_and(|last| {
            (tick.t_pos - last).abs() < MIN_LABEL_SPACING && (1.0 - tick.t_pos).abs() > 0.01
        });
        if crowded {
            tick.labeled = false;
        } else {
            last_labeled = Some(tick.t_pos);
        }
    }
}

pub use super::tick_label::{ScientificTick, TICK_BUF, format_scientific_tick};
