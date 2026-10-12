//! Cuts a font down to a set of characters with `allsorts`, keeping a Unicode
//! cmap so egui can still look the characters up.

use super::download::Error;
use allsorts::binary::read::ReadScope;
use allsorts::font::{Font, MatchingPresentation};
use allsorts::font_data::FontData;
use allsorts::subset::{CmapTarget, SubsetProfile};
use allsorts::tables::{FontTableProvider, NameTable};
use allsorts::tag;
use std::collections::BTreeSet;

/// `font` with only the glyphs of `chars` (and `.notdef`). Fails when the
/// font has no glyph for one of them.
pub fn subset(font: &[u8], chars: &BTreeSet<char>) -> Result<Vec<u8>, Error> {
    let data = ReadScope::new(font).read::<FontData<'_>>()?;
    let mut lookup = Font::new(data.table_provider(0)?)?;
    let mut glyphs = vec![0u16];
    let mut missing = String::new();
    for &ch in chars {
        let (glyph, _) = lookup.lookup_glyph_index(ch, MatchingPresentation::NotRequired, None);
        if glyph == 0 {
            missing.push(ch);
        } else if !glyphs.contains(&glyph) {
            glyphs.push(glyph);
        }
    }
    if !missing.is_empty() {
        return Err(format!("the font has no glyph for: {missing}").into());
    }
    let provider = data.table_provider(0)?;
    let bytes = allsorts::subset::subset(
        &provider,
        &glyphs,
        &SubsetProfile::Minimal,
        CmapTarget::Unicode,
    )?;
    Ok(bytes)
}

/// The font's copyright notice (name table entry 0).
pub fn copyright(font: &[u8]) -> Result<String, Error> {
    let data = ReadScope::new(font).read::<FontData<'_>>()?;
    let provider = data.table_provider(0)?;
    let name_data = provider.read_table_data(tag::NAME)?;
    let names = ReadScope::new(&name_data).read::<NameTable<'_>>()?;
    names
        .string_for_id(NameTable::COPYRIGHT_NOTICE)
        .ok_or_else(|| "the font has no copyright notice".into())
}
