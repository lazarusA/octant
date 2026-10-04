//! Matplotlib-derived families.

use super::notices::CLASSIC_NOTICE;
use super::urls::*;
use super::{Family, License, Source, picks};
use octant::utils::colormap::ColormapKind::Other;

pub const FAMILIES: &[Family] = &[
    Family {
        key: "matplotlib",
        name: "Matplotlib",
        license_id: "CC0-1.0",
        spdx: &["CC0-1.0"],
        license: License::Urls(&[BIDS_LICENSE]),
        notice: None,
        source: "https://github.com/BIDS/colormap",
        attribution: "Nathaniel J. Smith, Stefan van der Walt and Eric Firing",
        sources: &[Source::PyTables {
            url: BIDS_COLORMAPS,
            maps: picks::BIDS,
        }],
    },
    Family {
        key: "classic",
        name: "Classic",
        license_id: "Matplotlib / Apache-2.0 / MIT / BSD-3-Clause",
        spdx: &["Matplotlib", "Apache-2.0", "MIT", "BSD-3-Clause"],
        license: License::Urls(&[MPL_LICENSE, APACHE_2, WISTIA_LICENSE, VEGA_LICENSE]),
        notice: Some(CLASSIC_NOTICE),
        source: "https://github.com/matplotlib/matplotlib",
        attribution: "Matplotlib; Google (Turbo); Nunez et al. (Cividis); K. Moreland (Coolwarm); \
                      D. Green (Cubehelix); Wistia; Vega (tab palettes)",
        sources: &[
            Source::MatplotlibCm {
                url: MPL_CM,
                maps: picks::MPL_DATAD,
            },
            Source::PyTables {
                url: MPL_CM_LISTED,
                maps: picks::MPL_LISTED,
            },
            Source::CBytes {
                url: TURBO_C,
                array: "turbo_srgb_bytes",
                name: "turbo",
                kind: Other,
            },
        ],
    },
];
