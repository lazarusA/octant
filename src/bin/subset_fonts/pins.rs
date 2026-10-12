//! The pinned noto-cjk files, shared with `tools/cjk_slices` (which includes
//! this file by path, so it depends on nothing in Octant).

/// noto-cjk commit the fonts and license are read from.
pub const COMMIT: &str = "f8d157532fbfaeda587e826d4cd5b21a49186f7c";

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

/// An upstream font: the short code Octant names it by (`sc`, `jp`, `kr`),
/// where it lives and the name it is cached under.
pub struct Pin {
    pub code: &'static str,
    pub remote: Remote,
    pub file_name: &'static str,
}

/// The region-specific Regular weights the fonts are cut from.
pub const FONTS: [Pin; 3] = [
    Pin {
        code: "sc",
        remote: Remote {
            path: "Sans/SubsetOTF/SC/NotoSansSC-Regular.otf",
            md5: "b189a3afc31942fe81931222172cf48b",
        },
        file_name: "NotoSansSC-Regular.otf",
    },
    Pin {
        code: "jp",
        remote: Remote {
            path: "Sans/SubsetOTF/JP/NotoSansJP-Regular.otf",
            md5: "f82ef23cfbf8b46a234465b0151ee71f",
        },
        file_name: "NotoSansJP-Regular.otf",
    },
    Pin {
        code: "kr",
        remote: Remote {
            path: "Sans/SubsetOTF/KR/NotoSansKR-Regular.otf",
            md5: "2955ce7570641c69210186b840ccd72b",
        },
        file_name: "NotoSansKR-Regular.otf",
    },
];

/// The SIL Open Font License of the Noto Sans CJK fonts.
pub const LICENSE: Remote = Remote {
    path: "Sans/LICENSE",
    md5: "55719faa0112708e946b820b24b14097",
};
