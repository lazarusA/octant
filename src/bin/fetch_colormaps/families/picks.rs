//! Name tables selecting maps from multi-map upstream sources.

use super::Pick;
use octant::utils::colormap::ColormapKind::{Categorical, Cyclic, Diverging, Other, Sequential};

/// BIDS `colormaps.py`: the original perceptually uniform maps.
pub const BIDS: &[Pick] = &[
    ("_magma_data", "magma", Sequential),
    ("_inferno_data", "inferno", Sequential),
    ("_plasma_data", "plasma", Sequential),
    ("_viridis_data", "viridis", Sequential),
];

/// Matplotlib `_cm_listed.py` tables not covered by their original sources.
pub const MPL_LISTED: &[Pick] = &[
    ("_cividis_data", "cividis", Sequential),
    ("_twilight_data", "twilight", Cyclic),
];

/// Matplotlib `_cm.py` `datad` maps (ColorBrewer maps come from ColorBrewer itself).
pub const MPL_DATAD: &[Pick] = &[
    ("gray", "gray", Sequential),
    ("binary", "binary", Sequential),
    ("bone", "bone", Sequential),
    ("hot", "hot", Sequential),
    ("afmhot", "afmhot", Sequential),
    ("gist_heat", "gist_heat", Sequential),
    ("cool", "cool", Sequential),
    ("copper", "copper", Sequential),
    ("pink", "pink", Sequential),
    ("spring", "spring", Sequential),
    ("summer", "summer", Sequential),
    ("autumn", "autumn", Sequential),
    ("winter", "winter", Sequential),
    ("Wistia", "wistia", Sequential),
    ("ocean", "ocean", Sequential),
    ("gnuplot", "gnuplot", Sequential),
    ("gnuplot2", "gnuplot2", Sequential),
    ("cubehelix", "cubehelix", Sequential),
    ("CMRmap", "CMRmap", Sequential),
    ("coolwarm", "coolwarm", Diverging),
    ("bwr", "bwr", Diverging),
    ("seismic", "seismic", Diverging),
    ("hsv", "hsv", Cyclic),
    ("jet", "jet", Other),
    ("rainbow", "rainbow", Other),
    ("brg", "brg", Other),
    ("prism", "prism", Other),
    ("flag", "flag", Other),
    ("terrain", "terrain", Other),
    ("gist_earth", "gist_earth", Other),
    ("gist_stern", "gist_stern", Other),
    ("gist_ncar", "gist_ncar", Other),
    ("gist_rainbow", "gist_rainbow", Other),
    ("nipy_spectral", "nipy_spectral", Other),
    ("tab10", "tab10", Categorical),
    ("tab20", "tab20", Categorical),
    ("tab20b", "tab20b", Categorical),
    ("tab20c", "tab20c", Categorical),
];

/// Petroff's accessible color cycles, as contributed to Matplotlib.
pub const PETROFF: &[Pick] = &[
    ("_petroff6_data", "petroff6", Categorical),
    ("_petroff8_data", "petroff8", Categorical),
    ("_petroff10_data", "petroff10", Categorical),
];

/// seaborn `cm.py` lookup tables.
pub const SEABORN_LUTS: &[Pick] = &[
    ("_rocket_lut", "rocket", Sequential),
    ("_mako_lut", "mako", Sequential),
    ("_flare_lut", "flare", Sequential),
    ("_crest_lut", "crest", Sequential),
    ("_vlag_lut", "vlag", Diverging),
    ("_icefire_lut", "icefire", Diverging),
];

/// ColorSchemes.jl palettes extracted from public-domain paintings.
pub const PAINTINGS: &[&str] = &[
    "bosch_garden",
    "bosch_hell",
    "botticelli",
    "canaletto",
    "cezanne",
    "hokusai",
    "holbein",
    "klimt",
    "leonardo",
    "munch",
    "rembrandt",
    "starrynight",
    "vangogh",
    "vermeer",
];

/// cmyt modules at the pinned commit.
pub const CMYT: &[&str] = &[
    "algae",
    "apricity",
    "arbre",
    "dusk",
    "kelp",
    "octarine",
    "pastel",
    "pixel_blue",
    "pixel_green",
    "pixel_red",
    "xray",
];
