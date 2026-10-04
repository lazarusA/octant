//! Pinned upstream locations. Bump a commit here to refresh a family.

macro_rules! gh {
    ($repo:literal, $sha:literal, $path:literal) => {
        concat!(
            "https://raw.githubusercontent.com/",
            $repo,
            "/",
            $sha,
            "/",
            $path
        )
    };
}

pub const MPL_CM: &str = gh!(
    "matplotlib/matplotlib",
    "717459bbba42e8990601910b76ca365c0847a9f6",
    "lib/matplotlib/_cm.py"
);
pub const MPL_CM_LISTED: &str = gh!(
    "matplotlib/matplotlib",
    "717459bbba42e8990601910b76ca365c0847a9f6",
    "lib/matplotlib/_cm_listed.py"
);
pub const MPL_LICENSE: &str = gh!(
    "matplotlib/matplotlib",
    "717459bbba42e8990601910b76ca365c0847a9f6",
    "LICENSE/LICENSE"
);
pub const BIDS_COLORMAPS: &str = gh!(
    "BIDS/colormap",
    "bc549477db0c12b54a5928087552ad2cf274980f",
    "colormaps.py"
);
pub const BIDS_LICENSE: &str = gh!(
    "BIDS/colormap",
    "bc549477db0c12b54a5928087552ad2cf274980f",
    "LICENSE.txt"
);
pub const TURBO_C: &str = "https://gist.githubusercontent.com/mikhailov-work/6a308c20e494d9e0ccc29036b28faa7a/raw/turbo_colormap.c";
pub const APACHE_2: &str = "https://www.apache.org/licenses/LICENSE-2.0.txt";
pub const WISTIA_LICENSE: &str =
    "https://raw.githubusercontent.com/wistia/heatmap-palette/master/LICENSE";

pub const CMOCEAN_CM: &str = gh!(
    "matplotlib/cmocean",
    "59c35002c3aa5296b65d9646e52604c627441eb6",
    "cmocean/cm.py"
);
pub const CMOCEAN_RGB: &str = gh!(
    "matplotlib/cmocean",
    "59c35002c3aa5296b65d9646e52604c627441eb6",
    "cmocean/rgb/"
);
pub const CMOCEAN_LICENSE: &str = gh!(
    "matplotlib/cmocean",
    "59c35002c3aa5296b65d9646e52604c627441eb6",
    "LICENSE.txt"
);

pub const CRAMERI_ZIP: &str =
    "https://zenodo.org/api/records/8409685/files/ScientificColourMaps8.zip/content";

pub const COLORCET_INIT: &str = gh!(
    "holoviz/colorcet",
    "9f23c9659e81a3bd9b23c7bf874d5c9323f07c16",
    "colorcet/__init__.py"
);
pub const COLORCET_LICENSE: &str = gh!(
    "holoviz/colorcet",
    "9f23c9659e81a3bd9b23c7bf874d5c9323f07c16",
    "LICENSE.txt"
);

pub const MORELAND: &str = "https://www.kennethmoreland.com/color-advice/";
pub const PARAVIEW_LICENSE: &str = gh!(
    "Kitware/ParaView",
    "35fbf0b017f61190ae91c8bff257e807766e5b95",
    "Copyright.txt"
);
pub const VEGA_LICENSE: &str = gh!(
    "vega/vega",
    "9ef57269027e4bec7d36ca23aa5e9e196c96a8b7",
    "LICENSE"
);

pub const TOL_COLORS: &str = gh!(
    "Descanonge/tol_colors",
    "4834ac3e9652a9c5bcb7183197d8d8db71846288",
    "src/tol_colors/colors.json"
);
pub const TOL_LICENSE: &str = gh!(
    "Descanonge/tol_colors",
    "4834ac3e9652a9c5bcb7183197d8d8db71846288",
    "LICENSE"
);

pub const PETROFF_LICENSE: &str =
    "https://raw.githubusercontent.com/mpetroff/accessible-color-cycles/master/COPYING";

pub const BREWER_JSON: &str = gh!(
    "axismaps/colorbrewer",
    "7d135fc4e19eda73f2eb1bf55fcdf4a04fe4881f",
    "export/colorbrewer.json"
);
pub const BREWER_LICENSE: &str = gh!(
    "axismaps/colorbrewer",
    "7d135fc4e19eda73f2eb1bf55fcdf4a04fe4881f",
    "LICENCE.txt"
);

pub const CARTO_TS: &str = gh!(
    "CartoDB/CartoColor",
    "1a850e4713a12c68373f1df1a7d42ff7869ed49c",
    "src/carto.ts"
);
pub const CC_BY_4: &str = "https://creativecommons.org/licenses/by/4.0/legalcode.txt";

pub const CMASHER_TREE: &str = "https://api.github.com/repos/1313e/CMasher/git/trees/f4e76a7d8b93fd262d46e0c7e17c42815c3c8d94?recursive=1";
pub const CMASHER_RAW: &str = gh!(
    "1313e/CMasher",
    "f4e76a7d8b93fd262d46e0c7e17c42815c3c8d94",
    ""
);
pub const CMASHER_LICENSE: &str = gh!(
    "1313e/CMasher",
    "f4e76a7d8b93fd262d46e0c7e17c42815c3c8d94",
    "LICENSE"
);

pub const SEABORN_CM: &str = gh!(
    "mwaskom/seaborn",
    "f04b6cd5484267a0885d1fed068e99dff3a1b226",
    "seaborn/cm.py"
);
pub const SEABORN_PALETTES: &str = gh!(
    "mwaskom/seaborn",
    "f04b6cd5484267a0885d1fed068e99dff3a1b226",
    "seaborn/palettes.py"
);
pub const SEABORN_LICENSE: &str = gh!(
    "mwaskom/seaborn",
    "f04b6cd5484267a0885d1fed068e99dff3a1b226",
    "LICENSE.md"
);

pub const CMYT_BASE: &str = gh!(
    "yt-project/cmyt",
    "8ee346ad4670c67d1db0eb7a0b9843f9543103e6",
    "cmyt/colormaps/"
);
pub const CMYT_LICENSE: &str = gh!(
    "yt-project/cmyt",
    "8ee346ad4670c67d1db0eb7a0b9843f9543103e6",
    "LICENSE"
);

pub const METBREWER_R: &str = gh!(
    "BlakeRMills/MetBrewer",
    "58839e5ac7c7d604c8704581f7b201a29986b814",
    "R/PaletteCode.R"
);
pub const METBREWER_LICENSE: &str = gh!(
    "BlakeRMills/MetBrewer",
    "58839e5ac7c7d604c8704581f7b201a29986b814",
    "LICENSE.md"
);
pub const PNW_R: &str = gh!(
    "jakelawlor/PNWColors",
    "f16a14fcb0a1f93741e1acec221ce6391d93f0fa",
    "R/PNWColors.R"
);
pub const WES_R: &str = gh!(
    "karthik/wesanderson",
    "02e4129c185e9d3c49b05b580717f35be2eef5c3",
    "R/colors.R"
);
pub const GHIBLI_YML: &str = gh!(
    "ewenme/ghibli",
    "e030f0849bd0e2db7c89ca74173becceafdf268b",
    "inst/extdata/palettes.yml"
);
pub const LTC_R: &str = gh!(
    "loukesio/ltc-color-palettes",
    "6e69caa4c5601c419659887085196682db86639c",
    "R/ltc_functions.R"
);
pub const FEATHERS_R: &str = gh!(
    "shandiya/feathers",
    "09ab4c5cd1d88ce63b6076bc96ff28c8138cb139",
    "R/feathers.R"
);
pub const CATPPUCCIN_JSON: &str = gh!(
    "catppuccin/palette",
    "07d02aa110ef9eb7e7427afca5c73ba9cf7f8ebd",
    "palette.json"
);
pub const CATPPUCCIN_LICENSE: &str = gh!(
    "catppuccin/palette",
    "07d02aa110ef9eb7e7427afca5c73ba9cf7f8ebd",
    "LICENSE"
);
/// ColorSchemes.jl is used only as the original source of its own painting palettes.
pub const COLORSCHEMES_ALL: &str = gh!(
    "JuliaGraphics/ColorSchemes.jl",
    "0b51e30cb683fe38d1a07623db970ce283ba3cb3",
    "ColorSchemes/data/allcolorschemes.jl"
);
pub const COLORSCHEMES_LICENSE: &str = gh!(
    "JuliaGraphics/ColorSchemes.jl",
    "0b51e30cb683fe38d1a07623db970ce283ba3cb3",
    "ColorSchemes/LICENSE.md"
);
pub const GHIBLI_LICENSE: &str = gh!(
    "ewenme/ghibli",
    "e030f0849bd0e2db7c89ca74173becceafdf268b",
    "LICENSE.md"
);
pub const LTC_LICENSE: &str = gh!(
    "loukesio/ltc-color-palettes",
    "6e69caa4c5601c419659887085196682db86639c",
    "LICENSE.md"
);
pub const FEATHERS_LICENSE: &str = gh!(
    "shandiya/feathers",
    "09ab4c5cd1d88ce63b6076bc96ff28c8138cb139",
    "LICENSE.md"
);
pub const NORD_CSS: &str = gh!(
    "nordtheme/nord",
    "1cef71605416a222e57225b544540ce0fcec18d4",
    "src/nord.css"
);
pub const NORD_LICENSE: &str = gh!(
    "nordtheme/nord",
    "1cef71605416a222e57225b544540ce0fcec18d4",
    "license"
);
