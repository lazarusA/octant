//! Live sources: hosts that serve files without commits or releases to pin
//! (Moreland's and Krzywinski's sites). Each file is guarded by its MD5 alone.

use super::{Remote, Source};
use octant::utils::colormap::ColormapKind::{Diverging, Other, Sequential};

/// Live source, not pinnable; guarded by MD5.
macro_rules! moreland {
    ($dir:literal, $name:literal, $kind:expr, $md5:literal) => {
        Source::Table {
            url: Remote {
                url: concat!(
                    "https://www.kennethmoreland.com/color-advice/",
                    $dir,
                    "/",
                    $dir,
                    "-table-byte-0256.csv"
                ),
                md5: $md5,
            },
            name: $name,
            kind: $kind,
        }
    };
}

/// Live source, not pinnable; guarded by MD5.
macro_rules! krzywinski {
    ($n:literal, $md5:literal) => {
        Source::Krzywinski {
            url: Remote {
                url: concat!(
                    "https://mk.bcgsc.ca/colorblind/palettes/",
                    $n,
                    ".color.blindness.palette.txt"
                ),
                md5: $md5,
            },
            name: concat!("krzywinski", $n),
        }
    };
}

pub const MORELAND_TABLES: &[Source] = &[
    moreland!(
        "smooth-cool-warm",
        "smooth_cool_warm",
        Diverging,
        "a3f92777de289ea0e8815bb5337098a0"
    ),
    moreland!(
        "bent-cool-warm",
        "bent_cool_warm",
        Diverging,
        "28ead27d0b9b847c9eb008774bcb9615"
    ),
    moreland!(
        "black-body",
        "black_body",
        Sequential,
        "3d4caf6dbd114ab20f9b3a650d771b94"
    ),
    moreland!(
        "kindlmann",
        "kindlmann",
        Sequential,
        "4ba71ca00544364f3c435f75e30ab6df"
    ),
    moreland!(
        "extended-kindlmann",
        "extended_kindlmann",
        Sequential,
        "7a9ac4c569f193826b3cb5846afd3e7d"
    ),
    moreland!("fast", "fast", Other, "07dcd9f8cb685baa801bc3ea79230fd1"),
];

pub const KRZYWINSKI_PALETTES: &[Source] = &[
    krzywinski!("8", "5a583722ace6fb68aa84a02fe3567fc8"),
    krzywinski!("12", "aa0bcdaf07ab8d6c273e750c3057c2c3"),
    krzywinski!("15", "c93ca70b8a678073c2f3638abc0c3547"),
    krzywinski!("24", "2d8ab11e81967a7ab001e672f9e808d7"),
];
