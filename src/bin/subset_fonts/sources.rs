//! Where the fonts come from: the noto-cjk project, pinned to one commit.

use octant::ui::fonts::CjkFont;

/// noto-cjk commit the fonts and license are read from.
const COMMIT: &str = "f8d157532fbfaeda587e826d4cd5b21a49186f7c";

/// A file in the noto-cjk repository at [`COMMIT`], with its MD5.
pub struct Remote {
    pub path: &'static str,
    pub md5: &'static str,
}

impl Remote {
    pub fn url(&self) -> String {
        format!(
            "https://raw.githubusercontent.com/notofonts/noto-cjk/{COMMIT}/{}",
            self.path
        )
    }
}

/// An upstream font and the name it is cached under.
pub struct Source {
    pub remote: Remote,
    pub file_name: &'static str,
}

/// The region-specific Regular weight each font is cut from.
pub fn font(font: CjkFont) -> Source {
    match font {
        CjkFont::Sc => Source {
            remote: Remote {
                path: "Sans/SubsetOTF/SC/NotoSansSC-Regular.otf",
                md5: "b189a3afc31942fe81931222172cf48b",
            },
            file_name: "NotoSansSC-Regular.otf",
        },
        CjkFont::Jp => Source {
            remote: Remote {
                path: "Sans/SubsetOTF/JP/NotoSansJP-Regular.otf",
                md5: "f82ef23cfbf8b46a234465b0151ee71f",
            },
            file_name: "NotoSansJP-Regular.otf",
        },
        CjkFont::Kr => Source {
            remote: Remote {
                path: "Sans/SubsetOTF/KR/NotoSansKR-Regular.otf",
                md5: "2955ce7570641c69210186b840ccd72b",
            },
            file_name: "NotoSansKR-Regular.otf",
        },
    }
}

/// The SIL Open Font License of the Noto Sans CJK fonts.
pub const LICENSE: Remote = Remote {
    path: "Sans/LICENSE",
    md5: "55719faa0112708e946b820b24b14097",
};

/// `assets/fonts/LICENSE.md`: what the bundled files are, then the license.
pub fn license_notice(copyright: &str, license: &str) -> String {
    format!(
        "# Noto Sans CJK (SC, JP, KR)\n\n\
         The `noto-sans-*-ui.otf` and `noto-sans-*-names.otf` files are subsets of \
         Noto Sans SC, JP and KR Regular from https://github.com/notofonts/noto-cjk \
         (commit {COMMIT}), modified by Octant's `subset_fonts` tool to keep only the \
         characters its translations and language names use.\n\n\
         {copyright}\n\n\
         They are distributed under the SIL Open Font License, Version 1.1:\n\n\
         ```text\n{}\n```\n",
        license.trim_end()
    )
}
