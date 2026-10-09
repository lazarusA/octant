//! Hover card content: title, headline value, and coordinate rows.

use crate::ui::hover::composite::CompositeKind;
use crate::ui::hover::field::HoverField;
use crate::utils::stack_str::stack_str;
use egui::Color32;

/// The sampled value shown in the card headline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HoverValue {
    Scalar(f32),
    /// A composite pixel: the card names the band combination instead of a number.
    Composite(CompositeKind),
    NoData,
}

impl HoverValue {
    /// Classifies a raw sample; a composite pixel (packed display color) is named by
    /// `composite`. Only NaN is missing data: infinities are real values and display as `inf`.
    pub fn from_raw(raw: f32, composite: Option<CompositeKind>) -> Self {
        if raw.is_nan() {
            Self::NoData
        } else if let Some(kind) = composite {
            Self::Composite(kind)
        } else {
            Self::Scalar(raw)
        }
    }

    /// Formats the value into `buf` without allocating.
    pub fn format<'a>(&self, buf: &'a mut [u8; 32]) -> &'a str {
        match *self {
            Self::NoData => "No data",
            Self::Composite(kind) => kind.label(),
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
    /// Each drawn overlay's reading at the same cell, topmost first.
    pub layers: &'a [super::LayerValue<'a>],
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

/// Six significant digits, scientific for very large or small magnitudes, trailing
/// zeros trimmed.
fn format_scalar(buf: &mut [u8; 32], v: f32) -> &str {
    let mag = v.abs();
    let scientific = mag >= 1e6 || (mag < 1e-3 && v != 0.0);
    let (len, e_start, m_len) = {
        let s = if scientific {
            stack_str(buf, format_args!("{v:.4e}"))
        } else {
            // floor(log10) is the exponent of the leading digit: 287 -> 2, 0.0123 -> -2.
            let lead = if mag > 0.0 {
                mag.log10().floor() as i32
            } else {
                0
            };
            let decimals = (5 - lead).clamp(0, 7) as usize;
            stack_str(buf, format_args!("{v:.decimals$}"))
        };
        let e_start = s.find('e').unwrap_or(s.len());
        (s.len(), e_start, trim_fraction(&s[..e_start]).len())
    };
    // Shift the exponent (if any) left over the trimmed zeros.
    buf.copy_within(e_start..len, m_len);
    let out = std::str::from_utf8(&buf[..m_len + len - e_start]).unwrap_or_default();
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
