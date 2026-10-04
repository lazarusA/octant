//! Pinned upstream locations of the palette collections (R packages, themes and
//! ColorSchemes.jl paintings), each with the MD5 of its expected content.

use super::Remote;
use super::urls::gh;

pub const METBREWER_R: Remote = gh!(
    "BlakeRMills/MetBrewer",
    "58839e5ac7c7d604c8704581f7b201a29986b814",
    "R/PaletteCode.R",
    "3a5ba2298e2c776e89b3965a325f5491"
);
pub const METBREWER_LICENSE: Remote = gh!(
    "BlakeRMills/MetBrewer",
    "58839e5ac7c7d604c8704581f7b201a29986b814",
    "LICENSE.md",
    "3bedcaeda57cf8e31f791dd9e127eb0f"
);
pub const PNW_R: Remote = gh!(
    "jakelawlor/PNWColors",
    "f16a14fcb0a1f93741e1acec221ce6391d93f0fa",
    "R/PNWColors.R",
    "f0391684e8b471375e4e9809d9645efc"
);
pub const WES_R: Remote = gh!(
    "karthik/wesanderson",
    "02e4129c185e9d3c49b05b580717f35be2eef5c3",
    "R/colors.R",
    "d42140ec80063849a0a680f7996af321"
);
pub const GHIBLI_YML: Remote = gh!(
    "ewenme/ghibli",
    "e030f0849bd0e2db7c89ca74173becceafdf268b",
    "inst/extdata/palettes.yml",
    "c85c4e546db4cc30d5f23ed18bf11414"
);
pub const LTC_R: Remote = gh!(
    "loukesio/ltc-color-palettes",
    "6e69caa4c5601c419659887085196682db86639c",
    "R/ltc_functions.R",
    "fa87adbc7894ce4d4db2627a6af1ed06"
);
pub const FEATHERS_R: Remote = gh!(
    "shandiya/feathers",
    "09ab4c5cd1d88ce63b6076bc96ff28c8138cb139",
    "R/feathers.R",
    "7adb28dabdfd288c9dd1a7ee70da8ef8"
);
pub const CATPPUCCIN_JSON: Remote = gh!(
    "catppuccin/palette",
    "07d02aa110ef9eb7e7427afca5c73ba9cf7f8ebd",
    "palette.json",
    "3debf8f5717128c9b5e7abe6a6ecb6b8"
);
pub const CATPPUCCIN_LICENSE: Remote = gh!(
    "catppuccin/palette",
    "07d02aa110ef9eb7e7427afca5c73ba9cf7f8ebd",
    "LICENSE",
    "f41313b984a25905d1411a765434d319"
);
/// ColorSchemes.jl is used only as the original source of its own painting palettes.
pub const COLORSCHEMES_ALL: Remote = gh!(
    "JuliaGraphics/ColorSchemes.jl",
    "0b51e30cb683fe38d1a07623db970ce283ba3cb3",
    "ColorSchemes/data/allcolorschemes.jl",
    "87a238a5e0563535c189f85bbc0e4263"
);
pub const COLORSCHEMES_LICENSE: Remote = gh!(
    "JuliaGraphics/ColorSchemes.jl",
    "0b51e30cb683fe38d1a07623db970ce283ba3cb3",
    "ColorSchemes/LICENSE.md",
    "95fb0c585189d889a63cfe7bf24ef89a"
);
pub const GHIBLI_LICENSE: Remote = gh!(
    "ewenme/ghibli",
    "e030f0849bd0e2db7c89ca74173becceafdf268b",
    "LICENSE.md",
    "dc233d1d496ce8df981c77e6f310fc8c"
);
pub const LTC_LICENSE: Remote = gh!(
    "loukesio/ltc-color-palettes",
    "6e69caa4c5601c419659887085196682db86639c",
    "LICENSE.md",
    "3991545752c01e91fe86f6df017e1239"
);
pub const FEATHERS_LICENSE: Remote = gh!(
    "shandiya/feathers",
    "09ab4c5cd1d88ce63b6076bc96ff28c8138cb139",
    "LICENSE.md",
    "4cc5f5bd5a1a232c185031ca58e24839"
);
pub const NORD_CSS: Remote = gh!(
    "nordtheme/nord",
    "1cef71605416a222e57225b544540ce0fcec18d4",
    "src/nord.css",
    "c48b28b05ff4bbd345ff80c15876d191"
);
pub const NORD_LICENSE: Remote = gh!(
    "nordtheme/nord",
    "1cef71605416a222e57225b544540ce0fcec18d4",
    "license",
    "29f0ea5dfaeac14e77cf2fbd1776c99a"
);
