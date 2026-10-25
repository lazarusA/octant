//! Numbers in the UI language. Octant writes digits 0-9 in every language
//! and never groups thousands, so only the decimal separator changes: a
//! point in English, Chinese, Japanese and Korean, a comma in the others.
//! Dates stay ISO 8601 (`2020-01-15`), the same everywhere.
//!
//! Format a number as usual and localize it on the way out: [`Num`] wraps a
//! value written into a formatter, [`localizing`] wraps the formatter itself,
//! [`localize_bytes`] fixes a number already in a stack buffer, and
//! [`localize_string`] one in a `String`. Only apply them to text that is a
//! number (or numbers and fixed symbols): every `.` becomes the separator.
//! [`parse`] reads numbers typed in either style.

use super::{Language, language};
use std::fmt::{self, Display, Write};

/// The decimal separator of `lang`.
pub fn decimal_separator(lang: Language) -> char {
    match lang {
        Language::English | Language::ChineseSimp | Language::Japanese | Language::Korean => '.',
        _ => ',',
    }
}

/// A number written with the active language's decimal separator, e.g.
/// `Num(format_args!("{v:.2}"))`.
pub struct Num<T>(pub T);

impl<T: Display> Display for Num<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(localizing(f), "{}", self.0)
    }
}

/// `out` with every `.` written as the active language's decimal separator.
pub fn localizing<W: Write>(out: &mut W) -> Localizing<'_, W> {
    localizing_in(language(), out)
}

/// `out` with every `.` written as `lang`'s decimal separator.
pub fn localizing_in<W: Write>(lang: Language, out: &mut W) -> Localizing<'_, W> {
    Localizing {
        out,
        separator: decimal_separator(lang),
    }
}

/// See [`localizing`].
pub struct Localizing<'a, W> {
    out: &'a mut W,
    separator: char,
}

impl<W: Write> Write for Localizing<'_, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.separator == '.' {
            return self.out.write_str(s);
        }
        for (i, part) in s.split('.').enumerate() {
            if i > 0 {
                self.out.write_char(self.separator)?;
            }
            self.out.write_str(part)?;
        }
        Ok(())
    }
}

/// Rewrites the decimal points of formatted number text in place. Both
/// separators are one ASCII byte, so the text stays valid UTF-8.
pub fn localize_bytes(bytes: &mut [u8]) {
    localize_bytes_in(language(), bytes);
}

/// [`localize_bytes`] for `lang`.
pub fn localize_bytes_in(lang: Language, bytes: &mut [u8]) {
    let separator = decimal_separator(lang);
    if separator == '.' {
        return;
    }
    let separator = separator as u8;
    for byte in bytes.iter_mut().filter(|b| **b == b'.') {
        *byte = separator;
    }
}

/// [`localize_bytes`] on an owned `String`, without reallocating.
pub fn localize_string(text: String) -> String {
    let mut bytes = text.into_bytes();
    localize_bytes(&mut bytes);
    String::from_utf8(bytes).unwrap_or_default()
}

/// A number typed by the user. Full-width forms from CJK input methods
/// (`０．５`) read as ASCII, spaces are ignored and the Unicode minus reads as
/// `-`. In comma languages either separator is the decimal one; elsewhere a
/// point is, and a comma is accepted only as a thousands separator between
/// groups of three digits (`1,000.5`), so `0,5` is rejected rather than read
/// as 5.
pub fn parse(text: &str) -> Option<f64> {
    parse_in(language(), text)
}

/// [`parse`] for `lang`.
pub fn parse_in(lang: Language, text: &str) -> Option<f64> {
    let comma_decimal = decimal_separator(lang) == ',';
    let mut clean = String::with_capacity(text.len());
    for c in text.chars().map(halfwidth).filter(|c| !c.is_whitespace()) {
        match c {
            ',' if comma_decimal => clean.push('.'),
            '\u{2212}' => clean.push('-'),
            _ => clean.push(c),
        }
    }
    if !comma_decimal && clean.contains(',') {
        clean = ungroup(&clean)?;
    }
    clean.parse().ok()
}

/// The ASCII form of a full-width character (U+FF01..U+FF5E) or the
/// ideographic space; any other character unchanged.
pub(crate) fn halfwidth(c: char) -> char {
    match c {
        '\u{3000}' => ' ',
        '\u{FF01}'..='\u{FF5E}' => char::from_u32(u32::from(c) - 0xFEE0).unwrap_or(c),
        _ => c,
    }
}

/// `text` without its thousands separators, when its whole part is grouped
/// properly: one to three leading digits, then groups of exactly three.
fn ungroup(text: &str) -> Option<String> {
    let unsigned = text.trim_start_matches(['-', '+']);
    let whole_end = unsigned.find(['.', 'e', 'E']).unwrap_or(unsigned.len());
    let (whole, rest) = unsigned.split_at(whole_end);
    if rest.contains(',') {
        return None;
    }
    let mut groups = whole.split(',');
    let first = groups.next()?;
    let digits = |g: &str| g.bytes().all(|b| b.is_ascii_digit());
    let well_grouped = (1..=3).contains(&first.len())
        && digits(first)
        && groups.all(|g| g.len() == 3 && digits(g));
    well_grouped.then(|| text.replace(',', ""))
}
