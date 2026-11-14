//! What Octant reads of a GRIB2 message's sections itself: where they start, and whether
//! they are long enough for gribberish. gribberish reads template fields by indexing
//! (`data[64]`, `data[24..28]`) and panics on a section shorter than its template, which in
//! the browser (`panic = "abort"`) ends the data worker; such messages are refused first.

/// Where section `n` of the GRIB2 message `message` holds starts, and its stated length.
/// gribberish walks sections too but keeps their bytes to itself.
pub fn section_at(message: &[u8], n: u8) -> Option<(usize, usize)> {
    let mut at = 16usize;
    loop {
        let len = u32::from_be_bytes(message.get(at..at + 4)?.try_into().ok()?) as usize;
        if *message.get(at + 4)? == n {
            return Some((at, len));
        }
        at = at.checked_add(len.max(5))?;
    }
}

/// The most points a message may have: gribberish reserves its grid's coordinates while
/// opening and its values while decoding, never more than `MAX_DECODED_BYTES` of `f64`.
pub const MAX_POINTS: u64 =
    (crate::data::codecs::capped::MAX_DECODED_BYTES / std::mem::size_of::<f64>()) as u64;

/// The longest message read whole: its stated length is checked only against the file's
/// size, which a remote server states itself, and a message this long could not decode
/// within `MAX_POINTS` anyway.
pub const MAX_MESSAGE_BYTES: u64 = crate::data::codecs::capped::MAX_DECODED_BYTES as u64;

/// The largest number of data points `message` declares where gribberish reserves them: its
/// data representation section's (5, before decoding) and its grid section's (3, the
/// decoded values resized to it).
pub fn declared_points(message: &[u8]) -> Option<u64> {
    let count = |n: u8, field: usize| {
        let (at, _) = section_at(message, n)?;
        let count: [u8; 4] = message.get(at + field..at + field + 4)?.try_into().ok()?;
        Some(u64::from(u32::from_be_bytes(count)))
    };
    count(5, 5).max(count(3, 6))
}

/// The bytes of each section before its template that gribberish reads by indexing (its
/// length, number and fields up to the template number; the identification section's
/// reference time; the bitmap indicator; the data's first byte).
const HEADERS: &[(u8, usize)] = &[(1, 21), (3, 14), (4, 9), (5, 11), (6, 6), (7, 5)];

/// The bytes of each template gribberish reads, by template number (the highest byte its
/// template code reads, gribberish 1.10, also the WMO minimum lengths): grid definitions
/// (section 3) ...
const GRID: &[(u16, usize)] = &[(0, 72), (10, 72), (20, 65), (30, 81)];
/// ... product definitions (section 4) ...
const PRODUCT: &[(u16, usize)] = &[
    (0, 34),
    (1, 37),
    (2, 36),
    (5, 47),
    (6, 35),
    (8, 58),
    (9, 71),
    (10, 59),
    (11, 61),
    (12, 60),
    (103, 45),
    (104, 48),
    (107, 60),
];
/// ... and data representations (section 5).
const PACKING: &[(u16, usize)] = &[(0, 21), (2, 47), (3, 49), (40, 23), (41, 21), (42, 23)];

/// Whether each section of `message` that gribberish reads through a template is as long
/// as that template; a section missing from `message` (a head cut short) or with a template
/// gribberish does not read counts as fitting.
pub fn fit_templates(message: &[u8]) -> bool {
    let fits = |n: u8, template_at: usize, table: &[(u16, usize)]| {
        let Some((at, len)) = section_at(message, n) else {
            return true;
        };
        let template = message.get(at + template_at..at + template_at + 2);
        let template = template.map(|t| u16::from_be_bytes([t[0], t[1]]));
        let need = table.iter().find(|(t, _)| Some(*t) == template);
        need.is_none_or(|&(_, need)| len >= need)
    };
    let headers = (HEADERS.iter())
        .all(|&(n, need)| section_at(message, n).is_none_or(|(_, len)| len >= need));
    headers && fits(3, 12, GRID) && fits(4, 7, PRODUCT) && fits(5, 9, PACKING)
}
