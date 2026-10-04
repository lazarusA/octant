//! Scientific colormap families, each read from its original project.

use super::notices::{
    BREWER_NOTICE, CARTO_NOTICE, COLORCET_NOTICE, CRAMERI_COPYRIGHT, CRAMERI_NOTICE,
    CVD_FACTS_NOTICE, KRZYWINSKI_COPYRIGHT, KRZYWINSKI_NOTICE, MORELAND_NOTICE, PETROFF_NOTICE,
};
use super::urls::*;
use super::{Family, License, Source, live, picks};
use octant::utils::colormap::ColormapKind::Categorical;

pub const FAMILIES: &[Family] = &[
    Family {
        key: "cmocean",
        name: "cmocean",
        license_id: "MIT",
        spdx: &["MIT"],
        license: License::Urls(&[CMOCEAN_LICENSE]),
        notice: None,
        source: "https://github.com/matplotlib/cmocean",
        attribution: "Kristen M. Thyng et al., Oceanography 29(3), 2016",
        sources: &[Source::Cmocean {
            cm_py: CMOCEAN_CM,
            rgb_base: CMOCEAN_RGB,
        }],
    },
    Family {
        key: "scientific",
        name: "Scientific (Crameri)",
        license_id: "MIT",
        spdx: &["MIT"],
        license: License::Mit(CRAMERI_COPYRIGHT),
        notice: Some(CRAMERI_NOTICE),
        source: "https://doi.org/10.5281/zenodo.8409685",
        attribution: "Fabio Crameri, Scientific colour maps v8.0.1, doi:10.5281/zenodo.1243862",
        sources: &[Source::CrameriZip {
            url: CRAMERI_ZIP,
            cache: "ScientificColourMaps8.0.1.zip",
        }],
    },
    Family {
        key: "colorcet",
        name: "colorcet (CET)",
        license_id: "CC-BY-4.0",
        spdx: &["CC-BY-4.0"],
        license: License::Urls(&[COLORCET_LICENSE]),
        notice: Some(COLORCET_NOTICE),
        source: "https://github.com/holoviz/colorcet",
        attribution: "Peter Kovesi, CET Perceptually Uniform Colour Maps (colorcet.com), \
                      and Glasbey palettes, packaged by HoloViz colorcet; CC BY 4.0; \
                      quantized to 8-bit sRGB",
        sources: &[Source::Colorcet { url: COLORCET_INIT }],
    },
    Family {
        key: "moreland",
        name: "Moreland",
        license_id: "CC0-1.0 / BSD-3-Clause",
        spdx: &["CC0-1.0", "BSD-3-Clause"],
        license: License::Urls(&[PARAVIEW_LICENSE]),
        notice: Some(MORELAND_NOTICE),
        source: MORELAND,
        attribution: "Kenneth Moreland; Kindlmann, Reinhard & Creem (2002); \
                      Samsel, Scott & Moreland (Fast, 2024); ParaView (Kitware)",
        sources: live::MORELAND_TABLES,
    },
    Family {
        key: "tol",
        name: "Paul Tol",
        license_id: "BSD-3-Clause",
        spdx: &["BSD-3-Clause"],
        license: License::Urls(&[TOL_LICENSE]),
        notice: None,
        source: "https://github.com/Descanonge/tol_colors",
        attribution: "Paul Tol, Colour Schemes, SRON/EPS/TN/09-002 (2021)",
        sources: &[Source::Tol { url: TOL_COLORS }],
    },
    Family {
        key: "petroff",
        name: "Petroff",
        license_id: "MIT / Matplotlib",
        spdx: &["MIT", "Matplotlib"],
        license: License::Urls(&[PETROFF_LICENSE, MPL_LICENSE]),
        notice: Some(PETROFF_NOTICE),
        source: "https://github.com/mpetroff/accessible-color-cycles",
        attribution: "Matthew A. Petroff, Accessible Color Sequences for Data Visualization (2021)",
        sources: &[Source::PyTables {
            url: MPL_CM,
            maps: picks::PETROFF,
        }],
    },
    Family {
        key: "krzywinski",
        name: "Krzywinski",
        license_id: "MIT",
        spdx: &["MIT"],
        license: License::Mit(KRZYWINSKI_COPYRIGHT),
        notice: Some(KRZYWINSKI_NOTICE),
        source: "https://mk.bcgsc.ca/colorblind/palettes.mhtml",
        attribution: "Martin Krzywinski (8-color palette adapted from Bang Wong, Nature Methods 8:441, 2011)",
        sources: live::KRZYWINSKI_PALETTES,
    },
    Family {
        key: "colorblind",
        name: "Colorblind-safe",
        license_id: "Public domain (facts)",
        spdx: &["LicenseRef-Public-Domain-Facts"],
        license: License::Inline(CVD_FACTS_NOTICE),
        notice: None,
        source: "https://jfly.uni-koeln.de/color/",
        attribution: "Okabe & Ito (Color Universal Design), IBM Design Library",
        sources: &[
            Source::Hex {
                name: "okabe_ito",
                kind: Categorical,
                colors: &[
                    "#000000", "#E69F00", "#56B4E9", "#009E73", "#F0E442", "#0072B2", "#D55E00",
                    "#CC79A7",
                ],
            },
            Source::Hex {
                name: "ibm",
                kind: Categorical,
                colors: &["#648FFF", "#785EF0", "#DC267F", "#FE6100", "#FFB000"],
            },
        ],
    },
    Family {
        key: "colorbrewer",
        name: "ColorBrewer",
        license_id: "Apache-2.0",
        spdx: &["Apache-2.0"],
        license: License::Urls(&[BREWER_LICENSE]),
        notice: Some(BREWER_NOTICE),
        source: "https://colorbrewer2.org",
        attribution: "This product includes color specifications and designs developed by \
                      Cynthia Brewer (http://colorbrewer.org/)",
        sources: &[Source::ColorBrewer { url: BREWER_JSON }],
    },
    Family {
        key: "carto",
        name: "CARTOColors",
        license_id: "CC-BY-4.0",
        spdx: &["CC-BY-4.0"],
        license: License::Urls(&[CC_BY_4]),
        notice: Some(CARTO_NOTICE),
        source: "https://github.com/CartoDB/CartoColor",
        attribution: "CARTO, CARTOColors; CC BY 4.0; quantized to 8-bit sRGB",
        sources: &[Source::CartoTs { url: CARTO_TS }],
    },
    Family {
        key: "cmasher",
        name: "CMasher",
        license_id: "BSD-3-Clause",
        spdx: &["BSD-3-Clause"],
        license: License::Urls(&[CMASHER_LICENSE]),
        notice: None,
        source: "https://github.com/1313e/CMasher",
        attribution: "Ellert van der Velden, JOSS 5(46), 2020",
        sources: &[Source::Cmasher {
            tree_api: CMASHER_TREE,
            raw_base: CMASHER_RAW,
        }],
    },
    Family {
        key: "seaborn",
        name: "seaborn",
        license_id: "BSD-3-Clause",
        spdx: &["BSD-3-Clause"],
        license: License::Urls(&[SEABORN_LICENSE]),
        notice: None,
        source: "https://github.com/mwaskom/seaborn",
        attribution: "Michael L. Waskom",
        sources: &[
            Source::PyTables {
                url: SEABORN_CM,
                maps: picks::SEABORN_LUTS,
            },
            Source::SeabornPalettes {
                url: SEABORN_PALETTES,
            },
        ],
    },
    Family {
        key: "cmyt",
        name: "cmyt",
        license_id: "BSD-3-Clause",
        spdx: &["BSD-3-Clause"],
        license: License::Urls(&[CMYT_LICENSE]),
        notice: None,
        source: "https://github.com/yt-project/cmyt",
        attribution: "The yt project",
        sources: &[Source::Cmyt {
            base: CMYT_BASE,
            names: picks::CMYT,
        }],
    },
];
