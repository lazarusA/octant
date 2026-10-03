//! Hover card content: title, headline value, and coordinate rows.

use crate::ui::hover::field::HoverField;
use egui::Color32;
use std::io::Write;

/// The sampled value shown in the card headline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HoverValue {
    Scalar(f32),
    Rgb([u8; 3]),
    NoData,
}

impl HoverValue {
    /// Classifies a raw sample; RGB composites pack channels as `r | g << 8 | b << 16`.
    pub fn from_raw(raw: f32, is_rgb: bool) -> Self {
        if !raw.is_finite() {
            Self::NoData
        } else if is_rgb {
            let packed = raw.max(0.0) as u32;
            Self::Rgb([
                (packed & 0xFF) as u8,
                ((packed >> 8) & 0xFF) as u8,
                ((packed >> 16) & 0xFF) as u8,
            ])
        } else {
            Self::Scalar(raw)
        }
    }

    /// Formats the value into `buf` without allocating.
    pub fn format<'a>(&self, buf: &'a mut [u8; 32]) -> &'a str {
        match *self {
            Self::NoData => "No data",
            Self::Rgb([r, g, b]) => write_buf(buf, format_args!("{r}, {g}, {b}")),
            Self::Scalar(v) => format_scalar(buf, v),
        }
    }

    /// Units only make sense next to a number.
    pub fn shows_units(&self) -> bool {
        matches!(self, Self::Scalar(_))
    }
}

/// Everything the card paints, borrowed from the hover sample for one frame.
pub struct HoverCard<'a> {
    pub title: &'a str,
    pub value: HoverValue,
    pub units: &'a str,
    pub swatch: Color32,
    pub fields: &'a [HoverField],
}

impl<'a> HoverCard<'a> {
    /// The variable's `long_name` when available, otherwise its short name.
    pub fn title_for(name: &'a str, long_name: Option<&'a str>) -> &'a str {
        long_name
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .unwrap_or(name)
    }
}

/// About six significant digits (at most four decimals), scientific for very large or
/// small magnitudes, trailing zeros trimmed.
fn format_scalar(buf: &mut [u8; 32], v: f32) -> &str {
    let mag = v.abs();
    let scientific = mag >= 1e6 || (mag < 1e-3 && v != 0.0);
    let len = if scientific {
        write_len(buf, format_args!("{v:.4e}"))
    } else {
        let int_digits = if mag >= 1.0 { mag.log10() as i32 } else { 0 };
        let decimals = (5 - int_digits).clamp(0, 4) as usize;
        write_len(buf, format_args!("{v:.decimals$}"))
    };
    let (m_len, e_start) = {
        let s = std::str::from_utf8(&buf[..len]).unwrap_or("");
        let e_start = s.find('e').unwrap_or(len);
        (trim_fraction(&s[..e_start]).len(), e_start)
    };
    // Shift the exponent (if any) left over the trimmed zeros.
    buf.copy_within(e_start..len, m_len);
    let out = std::str::from_utf8(&buf[..m_len + len - e_start]).unwrap_or("");
    // "-0" reads as noise for values that round to zero.
    if out == "-0" { "0" } else { out }
}

fn trim_fraction(s: &str) -> &str {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        s
    }
}

fn write_len(buf: &mut [u8; 32], args: std::fmt::Arguments) -> usize {
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = cursor.write_fmt(args);
    cursor.position() as usize
}

fn write_buf<'a>(buf: &'a mut [u8; 32], args: std::fmt::Arguments) -> &'a str {
    let len = write_len(buf, args);
    std::str::from_utf8(&buf[..len]).unwrap_or("")
}
