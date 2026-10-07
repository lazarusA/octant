//! Opacity curves over the data position `t` in [0, 1], applied after the
//! colormap lookup. A curve is parsed from text, baked into one 256-entry row
//! (alpha channel) and registered as the last atlas row (see
//! [`super::registry::set_alpha_curve`]), so CPU and GPU sample the same values.
//!
//! Text forms, separated by commas, semicolons or spaces:
//! - values `0.1, 0.4, 0.3, 0.1`: evenly spaced from 0 to 1 (linear), or one
//!   equal bin each (step), which lines up with N categorical colorbar bins;
//! - stops `0:0, 0.2:1, 0.8:1, 1:0`: `position:alpha` pairs in increasing
//!   position, blended linearly or held until the next stop (step).

use super::lut::{LUT_SIZE, Lut, unit_to_u8};
use std::fmt;

/// Uniform `alpha_row` value when no opacity curve is active.
pub const NO_ALPHA_ROW: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlphaInterp {
    #[default]
    Linear,
    Step,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlphaCurve {
    Values(Vec<f32>),
    /// `[position, alpha]` pairs with non-decreasing positions.
    Stops(Vec<[f32; 2]>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlphaError {
    NotANumber(String),
    OutOfRange(f32),
    MixedForms,
    Unsorted,
}

impl fmt::Display for AlphaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotANumber(s) => write!(f, "'{s}' is not a number"),
            Self::OutOfRange(v) => write!(f, "{v} is outside 0..1"),
            Self::MixedForms => f.write_str("use either values or position:alpha stops"),
            Self::Unsorted => f.write_str("stop positions must increase"),
        }
    }
}

/// Built-in curves: label, text and interpolation.
pub const PRESETS: [(&str, &str, AlphaInterp); 6] = [
    ("Ramp up", "0, 1", AlphaInterp::Linear),
    ("Ramp down", "1, 0", AlphaInterp::Linear),
    ("Peak", "0:0, 0.5:1, 1:0", AlphaInterp::Linear),
    ("Sink", "0:1, 0.5:0, 1:1", AlphaInterp::Linear),
    ("Hide low", "0:0, 0.1:1", AlphaInterp::Step),
    ("Hide center", "0:1, 0.4:0, 0.6:1", AlphaInterp::Step),
];

/// Parses curve text; blank text is no curve.
pub fn parse(text: &str) -> Result<Option<AlphaCurve>, AlphaError> {
    let mut values = Vec::new();
    let mut stops = Vec::new();

    // Normalize whitespace around ':' so "0.2 : 0.8" or "0.2: 0.8" becomes "0.2:0.8"
    let mut normalized = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ':' {
            while normalized.ends_with(|w: char| w.is_whitespace()) {
                normalized.pop();
            }
            normalized.push(':');
            while chars.peek().is_some_and(|&p| p.is_whitespace()) {
                chars.next();
            }
        } else {
            normalized.push(c);
        }
    }

    let tokens = normalized
        .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .filter(|s| !s.is_empty());
    for token in tokens {
        match token.split_once(':') {
            Some((pos, alpha)) => stops.push([unit(pos)?, unit(alpha)?]),
            None => values.push(unit(token)?),
        }
    }
    match (values.is_empty(), stops.is_empty()) {
        (true, true) => Ok(None),
        (false, false) => Err(AlphaError::MixedForms),
        (false, true) => Ok(Some(AlphaCurve::Values(values))),
        (true, false) if stops.windows(2).any(|w| w[1][0] < w[0][0]) => Err(AlphaError::Unsorted),
        (true, false) => Ok(Some(AlphaCurve::Stops(stops))),
    }
}

/// A number in [0, 1].
fn unit(s: &str) -> Result<f32, AlphaError> {
    let v: f32 = s
        .parse()
        .map_err(|_| AlphaError::NotANumber(s.to_string()))?;
    if (0.0..=1.0).contains(&v) {
        Ok(v)
    } else {
        Err(AlphaError::OutOfRange(v))
    }
}

/// Bakes a curve into a LUT whose alpha channel holds the curve (RGB white).
pub fn bake(curve: &AlphaCurve, interp: AlphaInterp) -> Box<Lut> {
    let mut lut = Box::new([[255; 4]; LUT_SIZE]);
    for (i, slot) in lut.iter_mut().enumerate() {
        let a = match curve {
            AlphaCurve::Values(v) => from_values(v, i, interp),
            AlphaCurve::Stops(s) => from_stops(s, i as f32 / (LUT_SIZE - 1) as f32, interp),
        };
        slot[3] = unit_to_u8(a);
    }
    lut
}

fn from_values(v: &[f32], i: usize, interp: AlphaInterp) -> f32 {
    let n = v.len();
    if n <= 1 {
        return v.first().copied().unwrap_or(1.0);
    }
    if interp == AlphaInterp::Step {
        // Bin by the texel's position `t = i / 255`, the scale samples use, so
        // bin centers `(b + 0.5) / n` land inside bin `b` (exact up to 128 bins).
        let t = i as f32 / (LUT_SIZE - 1) as f32;
        return v[((t * n as f32) as usize).min(n - 1)];
    }
    let pos = i as f32 / (LUT_SIZE - 1) as f32 * (n - 1) as f32;
    let j = (pos.floor() as usize).min(n - 2);
    lerp(v[j], v[j + 1], pos - j as f32)
}

fn from_stops(s: &[[f32; 2]], x: f32, interp: AlphaInterp) -> f32 {
    // Stops at or before x; equal positions make a hard jump.
    let k = s.partition_point(|p| p[0] <= x);
    let Some(first) = s.first() else {
        return 1.0;
    };
    if k == 0 {
        return first[1];
    }
    let [p0, a0] = s[k - 1];
    if interp == AlphaInterp::Step || k == s.len() {
        return a0;
    }
    let [p1, a1] = s[k];
    lerp(a0, a1, (x - p0) / (p1 - p0))
}

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a + (b - a) * f
}
