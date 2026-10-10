//! Tick values in clean decimal or concise scientific notation, written
//! straight into a formatter so per-frame code can use a stack buffer.

/// Formats tick values cleanly using integer/decimal or concise scientific notation.
pub fn format_scientific_tick(val: f32) -> String {
    ScientificTick(val).to_string()
}

/// A tick value formatted like `format_scientific_tick`, written straight
/// into a formatter so per-frame UI code can use a stack buffer
/// (`stack_str(&mut buf, format_args!("{}", ScientificTick(v)))`).
#[derive(Clone, Copy)]
pub struct ScientificTick(pub f32);

/// Room for a formatted `ScientificTick` (the longest, e.g. `-1.23e-45`, is 9 bytes).
pub const TICK_BUF: usize = 16;

impl std::fmt::Display for ScientificTick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.0;
        let abs_val = val.abs();
        if abs_val == 0.0 {
            f.write_str("0")
        } else if !(0.001..10000.0).contains(&abs_val) {
            let mut buf = [0u8; 32];
            let s = crate::utils::stack_str(&mut buf, format_args!("{val:.2e}"));
            match s.split_once('e') {
                Some((mantissa, exponent)) => {
                    let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
                    write!(f, "{mantissa}e{exponent}")
                }
                None => f.write_str(s),
            }
        } else if val.fract().abs() < 1e-5 {
            write!(f, "{val:.0}")
        } else if (val * 10.0).fract().abs() < 1e-5 {
            write!(f, "{val:.1}")
        } else {
            write!(f, "{val:.2}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ScientificTick, TICK_BUF, format_scientific_tick};

    #[test]
    fn scientific_tick_fits_its_stack_buffer() {
        for val in [
            -1.23e-45,
            -f32::MAX,
            f32::MIN_POSITIVE,
            -9999.99,
            f32::NAN,
            f32::NEG_INFINITY,
        ] {
            let mut buf = [0u8; TICK_BUF];
            let text = crate::utils::stack_str(&mut buf, format_args!("{}", ScientificTick(val)));
            assert_eq!(text, format_scientific_tick(val));
        }
    }
}
