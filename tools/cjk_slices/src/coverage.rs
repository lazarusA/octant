//! The characters a font can draw within the scripts Octant slices: Hangul
//! jamo, CJK radicals through Hangul syllables, compatibility ideographs,
//! vertical and full-width forms, and the supplementary ideographs.

use crate::download::Error;
use allsorts::binary::read::ReadScope;
use allsorts::font::{Font, MatchingPresentation};
use allsorts::font_data::FontData;

const RANGES: [(u32, u32); 6] = [
    (0x1100, 0x11FF),
    (0x2E80, 0xD7FF),
    (0xF900, 0xFAFF),
    (0xFE30, 0xFE4F),
    (0xFF00, 0xFFEF),
    (0x20000, 0x3134F),
];

/// Codepoints of `RANGES` that `font` has a glyph for, ascending.
pub fn codepoints(font: &[u8]) -> Result<Vec<u32>, Error> {
    let data = ReadScope::new(font).read::<FontData<'_>>()?;
    let mut lookup = Font::new(data.table_provider(0)?)?;
    Ok(RANGES
        .iter()
        .flat_map(|&(first, last)| first..=last)
        .filter(|&c| {
            char::from_u32(c).is_some_and(|ch| {
                lookup
                    .lookup_glyph_index(ch, MatchingPresentation::NotRequired, None)
                    .0
                    != 0
            })
        })
        .collect())
}
