//! Where the fonts come from: the noto-cjk project, pinned to one commit
//! (`pins.rs`), and the license notice written next to them.

use super::pins::{COMMIT, FONTS, Pin};
use octant::ui::fonts::CjkFont;

/// The pinned upstream font `font` is cut from.
pub fn font(font: CjkFont) -> &'static Pin {
    let code = match font {
        CjkFont::Sc => "sc",
        CjkFont::Jp => "jp",
        CjkFont::Kr => "kr",
    };
    FONTS
        .iter()
        .find(|pin| pin.code == code)
        .unwrap_or(&FONTS[0])
}

/// `assets/fonts/LICENSE.md`: what the bundled files are, then the license.
pub fn license_notice(copyright: &str, license: &str) -> String {
    format!(
        "# Noto Sans CJK (SC, JP, KR)\n\n\
         The `noto-sans-*-ui.otf` and `noto-sans-*-names.otf` files are subsets of \
         Noto Sans SC, JP and KR Regular from https://github.com/notofonts/noto-cjk \
         (commit {COMMIT}), modified by Octant's `subset_fonts` tool to keep only the \
         characters its translations and language names use. The web build also \
         serves slices of the same fonts (`fonts/cjk/`, built by `tools/cjk_slices`) \
         for text in datasets.\n\n\
         {copyright}\n\n\
         They are distributed under the SIL Open Font License, Version 1.1:\n\n\
         ```text\n{}\n```\n",
        license.trim_end()
    )
}
