//! Pinned upstream locations, each with the MD5 of its expected content. Bump a
//! commit (and its checksum) here to refresh a family. Sources that cannot be
//! pinned to a commit are live and guarded by their MD5 alone.

use super::Remote;

macro_rules! gh {
    ($repo:literal, $sha:literal, $path:literal, $md5:literal) => {
        Remote {
            url: gh!($repo, $sha, $path),
            md5: $md5,
        }
    };
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
pub(super) use gh;

pub const MPL_CM: Remote = gh!(
    "matplotlib/matplotlib",
    "717459bbba42e8990601910b76ca365c0847a9f6",
    "lib/matplotlib/_cm.py",
    "693a5814fccc8e57833daf9c6f67ff4c"
);
pub const MPL_CM_LISTED: Remote = gh!(
    "matplotlib/matplotlib",
    "717459bbba42e8990601910b76ca365c0847a9f6",
    "lib/matplotlib/_cm_listed.py",
    "60854d42474035bde0052b9b508262b1"
);
pub const MPL_LICENSE: Remote = gh!(
    "matplotlib/matplotlib",
    "717459bbba42e8990601910b76ca365c0847a9f6",
    "LICENSE/LICENSE",
    "afec61498aa5f0c45936687da9a53d74"
);
pub const BIDS_COLORMAPS: Remote = gh!(
    "BIDS/colormap",
    "bc549477db0c12b54a5928087552ad2cf274980f",
    "colormaps.py",
    "827ef14fb6cc491ff64502c38ae6336e"
);
pub const BIDS_LICENSE: Remote = gh!(
    "BIDS/colormap",
    "bc549477db0c12b54a5928087552ad2cf274980f",
    "LICENSE.txt",
    "8ca9a3242600150c0fd0812ad2eb44c9"
);
pub const TURBO_C: Remote = Remote {
    url: "https://gist.githubusercontent.com/mikhailov-work/6a308c20e494d9e0ccc29036b28faa7a/raw/a438da918f140eac8775be5e302f63795d9e28c3/turbo_colormap.c",
    md5: "af5b158e9bfb36599e7f9d75004d36fa",
};
/// Live source, not pinnable; guarded by MD5.
pub const APACHE_2: Remote = Remote {
    url: "https://www.apache.org/licenses/LICENSE-2.0.txt",
    md5: "3b83ef96387f14655fc854ddc3c6bd57",
};
pub const WISTIA_LICENSE: Remote = gh!(
    "wistia/heatmap-palette",
    "5647641101bbe6e6b3ac409782e5a99ee3e5f65d",
    "LICENSE",
    "7f60848e14fdcc2f93961a3d7f7a10ef"
);

pub const CMOCEAN_CM: Remote = gh!(
    "matplotlib/cmocean",
    "59c35002c3aa5296b65d9646e52604c627441eb6",
    "cmocean/cm.py",
    "82aa57b5da6612d2e06f545bde96387f"
);
pub const CMOCEAN_RGB: &str = gh!(
    "matplotlib/cmocean",
    "59c35002c3aa5296b65d9646e52604c627441eb6",
    "cmocean/rgb/"
);
pub const CMOCEAN_LICENSE: Remote = gh!(
    "matplotlib/cmocean",
    "59c35002c3aa5296b65d9646e52604c627441eb6",
    "LICENSE.txt",
    "769ff1cb302d0c59283e316b8cce1067"
);

/// Zenodo release archive (v8.0.1); also cached on disk under its MD5.
pub const CRAMERI_ZIP: Remote = Remote {
    url: "https://zenodo.org/api/records/8409685/files/ScientificColourMaps8.zip/content",
    md5: "e63c1d2dbed7eb9c8a177ab8da37365b",
};

pub const COLORCET_INIT: Remote = gh!(
    "holoviz/colorcet",
    "9f23c9659e81a3bd9b23c7bf874d5c9323f07c16",
    "colorcet/__init__.py",
    "d343ccb7ac28220a764f24cb60f3e0eb"
);
pub const COLORCET_LICENSE: Remote = gh!(
    "holoviz/colorcet",
    "9f23c9659e81a3bd9b23c7bf874d5c9323f07c16",
    "LICENSE.txt",
    "b443afaf131aa64a6644f9bd3383f208"
);

pub const MORELAND: &str = "https://www.kennethmoreland.com/color-advice/";
pub const PARAVIEW_LICENSE: Remote = gh!(
    "Kitware/ParaView",
    "35fbf0b017f61190ae91c8bff257e807766e5b95",
    "Copyright.txt",
    "14a55075f4d21fb0263e55a3e4508b29"
);
pub const VEGA_LICENSE: Remote = gh!(
    "vega/vega",
    "9ef57269027e4bec7d36ca23aa5e9e196c96a8b7",
    "LICENSE",
    "0c7c8bfe5f44620511cd5bc5679d4790"
);

pub const TOL_COLORS: Remote = gh!(
    "Descanonge/tol_colors",
    "4834ac3e9652a9c5bcb7183197d8d8db71846288",
    "src/tol_colors/colors.json",
    "562840a90443d8e518082f124d9d984b"
);
pub const TOL_LICENSE: Remote = gh!(
    "Descanonge/tol_colors",
    "4834ac3e9652a9c5bcb7183197d8d8db71846288",
    "LICENSE",
    "1ed85fba0f2f94549583c12fe991951c"
);

pub const PETROFF_LICENSE: Remote = gh!(
    "mpetroff/accessible-color-cycles",
    "a7e0fcfed28f4ba6393a6e99bc7ab8694e5253fe",
    "COPYING",
    "8ab1ba358505b1a848c7b882305db057"
);

pub const BREWER_JSON: Remote = gh!(
    "axismaps/colorbrewer",
    "7d135fc4e19eda73f2eb1bf55fcdf4a04fe4881f",
    "export/colorbrewer.json",
    "3eb90704b30adb1a3d620a0441673c46"
);
pub const BREWER_LICENSE: Remote = gh!(
    "axismaps/colorbrewer",
    "7d135fc4e19eda73f2eb1bf55fcdf4a04fe4881f",
    "LICENCE.txt",
    "e3fc50a88d0a364313df4b21ef20c29e"
);

pub const CARTO_TS: Remote = gh!(
    "CartoDB/CartoColor",
    "1a850e4713a12c68373f1df1a7d42ff7869ed49c",
    "src/carto.ts",
    "8035c3f4612e3c9c1e03bd0c7a1ceba9"
);
/// Live source, not pinnable; guarded by MD5.
pub const CC_BY_4: Remote = Remote {
    url: "https://creativecommons.org/licenses/by/4.0/legalcode.txt",
    md5: "2ab724713fdaf49e4523c4503bfd068d",
};

/// The tree of the pinned commit, listing the `.jscm` files read from `CMASHER_RAW`.
pub const CMASHER_TREE: Remote = Remote {
    url: "https://api.github.com/repos/1313e/CMasher/git/trees/f4e76a7d8b93fd262d46e0c7e17c42815c3c8d94?recursive=1",
    md5: "383cfcf1d403678179596b44aede786a",
};
pub const CMASHER_RAW: &str = gh!(
    "1313e/CMasher",
    "f4e76a7d8b93fd262d46e0c7e17c42815c3c8d94",
    ""
);
pub const CMASHER_LICENSE: Remote = gh!(
    "1313e/CMasher",
    "f4e76a7d8b93fd262d46e0c7e17c42815c3c8d94",
    "LICENSE",
    "c5fa430976f8c7931e8d22fa59a55091"
);

pub const SEABORN_CM: Remote = gh!(
    "mwaskom/seaborn",
    "f04b6cd5484267a0885d1fed068e99dff3a1b226",
    "seaborn/cm.py",
    "38506d62c9e7b30071c3623e2a01991e"
);
pub const SEABORN_PALETTES: Remote = gh!(
    "mwaskom/seaborn",
    "f04b6cd5484267a0885d1fed068e99dff3a1b226",
    "seaborn/palettes.py",
    "5d51b733be2f2dc22ed3db3f2559578f"
);
pub const SEABORN_LICENSE: Remote = gh!(
    "mwaskom/seaborn",
    "f04b6cd5484267a0885d1fed068e99dff3a1b226",
    "LICENSE.md",
    "bff71107d52061931e88f61561c06dd4"
);

pub const CMYT_BASE: &str = gh!(
    "yt-project/cmyt",
    "8ee346ad4670c67d1db0eb7a0b9843f9543103e6",
    "cmyt/colormaps/"
);
pub const CMYT_LICENSE: Remote = gh!(
    "yt-project/cmyt",
    "8ee346ad4670c67d1db0eb7a0b9843f9543103e6",
    "LICENSE",
    "0533ae68ec1967529a2bb9ec2b90fc41"
);
