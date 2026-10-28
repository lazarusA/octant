//! Zero-allocation tick marks and time/numeric domain formatting.

use std::io::Write;

/// Zero-allocation tick mark storing its formatted label in a fixed stack buffer.
#[derive(Clone, Copy)]
pub struct TickMark {
    pub val: f64,
    /// Position along the visible axis: 0 at its start, 1 at its end.
    pub fract: f32,
    len: u8,
    buf: [u8; 32],
}

impl TickMark {
    pub const EMPTY: Self = Self {
        val: 0.0,
        fract: 0.0,
        len: 0,
        buf: [0u8; 32],
    };

    pub fn new_with_context(
        val: f64,
        step: f64,
        total_span: f64,
        title: &str,
        units: Option<&str>,
    ) -> Self {
        let time = is_time_axis(title, units);
        Self::on_axis(val, step, total_span, title, units, time)
    }

    /// A tick on an axis already known to be a time axis or not ([`is_time_axis`]).
    pub(super) fn on_axis(
        val: f64,
        step: f64,
        total_span: f64,
        title: &str,
        units: Option<&str>,
        time: bool,
    ) -> Self {
        let mut mark = Self { val, ..Self::EMPTY };
        if !(time && mark.try_format_time(total_span, title, units)) {
            mark.format(step);
        }
        mark
    }

    fn try_format_time(&mut self, total_span: f64, title: &str, units: Option<&str>) -> bool {
        let mut cursor = std::io::Cursor::new(&mut self.buf[..]);

        if let Some(units_str) = units {
            let clean = units_str.trim();
            if let Some((unit_part, ref_date)) = crate::utils::units::split_since(clean) {
                let (y, m, d) =
                    crate::utils::units::parse_iso_date(ref_date).unwrap_or((1970, 1, 1));
                let scale_ms =
                    crate::utils::units::unit_to_milliseconds(unit_part).unwrap_or(1_000) as f64;
                let total_hours = (self.val * scale_ms / 3_600_000.0).round() as i64;
                let (res_y, res_m, res_d) =
                    crate::utils::units::add_days_to_date(y, m, d, total_hours.div_euclid(24));
                let span_days = (total_span.abs() * scale_ms) / 86_400_000.0;
                write_date_span(
                    &mut cursor,
                    span_days,
                    res_y,
                    res_m,
                    res_d,
                    total_hours.rem_euclid(24) as usize,
                );
                self.len = cursor.position() as u8;
                return self.len > 0;
            }

            if let Some((y, m, d)) = crate::utils::units::parse_iso_date(clean) {
                let (res_y, res_m, res_d) =
                    crate::utils::units::add_days_to_date(y, m, d, self.val.round() as i64);
                write_date_span(&mut cursor, total_span.abs(), res_y, res_m, res_d, 0);
                self.len = cursor.position() as u8;
                return self.len > 0;
            }

            let lower = clean.to_ascii_lowercase();
            if matches!(lower.as_str(), "h" | "hr" | "hours" | "hour") {
                let _ = write!(cursor, "{:.0} h", self.val);
                self.len = cursor.position() as u8;
                return self.len > 0;
            }
            if matches!(lower.as_str(), "d" | "day" | "days") {
                let _ = write!(cursor, "{:.0} d", self.val);
                self.len = cursor.position() as u8;
                return self.len > 0;
            }
        }

        if self.val.abs() > 1e8 && self.val.abs() < 1e13 {
            let total_secs = if self.val.abs() > 1e11 {
                (self.val / 1000.0).round() as i64
            } else {
                self.val.round() as i64
            };
            let total_hours = total_secs.div_euclid(3600);
            let (res_y, res_m, res_d) =
                crate::utils::units::add_days_to_date(1970, 1, 1, total_hours.div_euclid(24));
            let span_secs = if total_span.abs() > 1e11 {
                total_span.abs() / 1000.0
            } else {
                total_span.abs()
            };
            write_date_span(
                &mut cursor,
                span_secs / 86400.0,
                res_y,
                res_m,
                res_d,
                total_hours.rem_euclid(24) as usize,
            );
            self.len = cursor.position() as u8;
            return self.len > 0;
        }

        if title.eq_ignore_ascii_case("step") || title.eq_ignore_ascii_case("timestep") {
            let _ = write!(cursor, "t={:.0}", self.val);
            self.len = cursor.position() as u8;
            return self.len > 0;
        }

        false
    }

    fn format(&mut self, step: f64) {
        let mut cursor = std::io::Cursor::new(&mut self.buf[..]);
        let abs_val = self.val.abs();
        let _ = if abs_val > 0.0 && !(1e-3..1e5).contains(&abs_val) {
            write!(cursor, "{:.2e}", self.val)
        } else if step.abs() < 1.0 {
            // Enough decimals to tell steps apart: 0.5 -> 1, 0.05 -> 2.
            let decimals = ((-step.abs().log10() - 1e-9).ceil().max(0.0) as usize).min(6);
            write!(cursor, "{:.*}", decimals, self.val)
        } else if self.val.fract().abs() < 1e-6 {
            write!(cursor, "{:.0}", self.val)
        } else {
            write!(cursor, "{:.2}", self.val)
        };
        self.len = cursor.position() as u8;
        crate::i18n::number::localize_bytes(&mut self.buf[..self.len as usize]);
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("")
    }
}

fn write_date_span(
    cursor: &mut std::io::Cursor<&mut [u8]>,
    span_days: f64,
    y: usize,
    m: usize,
    d: usize,
    hour: usize,
) {
    let _ = if span_days > 730.0 {
        write!(cursor, "{:04}", y)
    } else if span_days > 60.0 {
        write!(cursor, "{:04}-{:02}", y, m)
    } else if span_days > 2.0 {
        write!(cursor, "{:02}-{:02}", m, d)
    } else {
        write!(cursor, "{:02}:{:02}", hour, 0)
    };
}

/// Whether an axis holds times: its title names time, or its units are CF time units.
pub(super) fn is_time_axis(title: &str, units: Option<&str>) -> bool {
    crate::data::coordinates::naming::is_animated_time_name(title)
        || units.is_some_and(crate::utils::units::is_cf_time_unit)
}
