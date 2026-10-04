//! Curated, permissively licensed colormap families.
//!
//! Only families whose licenses are compatible with Octant's `MIT OR Apache-2.0`
//! are bundled: public-domain dedications and permissive licenses whose
//! conditions are met by keeping the notice and crediting the authors.
//! Copyleft and non-commercial licenses are refused by [`check_compatible`].

mod classic;
mod live;
mod notices;
mod palette_urls;
mod palettes;
mod picks;
mod scientific;
mod urls;

use octant::utils::colormap::ColormapKind;

/// Licenses (SPDX ids) that can be redistributed inside an `MIT OR Apache-2.0`
/// application. `Matplotlib` is the PSF-based, BSD-compatible Matplotlib license.
/// `CC-BY-4.0` requires visible attribution (creator, license, link, and a note
/// of modifications), provided by `LICENSES.md` and the About dialog credits.
/// `LicenseRef-Public-Domain-Facts` marks short published lists of color values
/// with no license claimed: facts, not copyrightable expression, still credited.
pub const COMPATIBLE_LICENSES: &[&str] = &[
    "CC0-1.0",
    "MIT",
    "BSD-3-Clause",
    "Apache-2.0",
    "Matplotlib",
    "CC-BY-4.0",
    "LicenseRef-Public-Domain-Facts",
];

/// A downloaded file and the MD5 of its expected content. Fresh downloads and
/// cached copies are both verified, so upstream drift is an error, not a silent change.
#[derive(Clone, Copy)]
pub struct Remote {
    pub url: &'static str,
    pub md5: &'static str,
}

/// Where a family's license text comes from.
pub enum License {
    /// The official license file(s) published by the upstream project(s).
    Urls(&'static [Remote]),
    /// Text kept here because upstream ships no license file.
    Inline(&'static str),
    /// Packages declaring MIT without the full text: the MIT text with this copyright line.
    Mit(&'static str),
}

/// A map selected from a multi-map source: (upstream name, Octant name, kind).
pub type Pick = (&'static str, &'static str, ColormapKind);

/// Where a family's colormaps come from. Every URL points at the original
/// project, pinned to a commit or release where the host allows it, and every
/// [`Remote`] is checked against its MD5. Per-map files listed upstream
/// (`rgb_base`, `raw_base`, `base`) must be commit-pinned.
pub enum Source {
    /// Matplotlib `_cm.py` `datad` entries (segment tables, functions, lists, listed).
    MatplotlibCm {
        url: Remote,
        maps: &'static [Pick],
    },
    /// Python float tables `var = [[r, g, b], ...]` (Matplotlib, BIDS, seaborn).
    PyTables {
        url: Remote,
        maps: &'static [Pick],
    },
    /// A C source with an `unsigned char <array>[256][3]` table (Turbo).
    CBytes {
        url: Remote,
        array: &'static str,
        name: &'static str,
        kind: ColormapKind,
    },
    /// One numeric table (CSV or whitespace separated, 0–1 or 0–255).
    Table {
        url: Remote,
        name: &'static str,
        kind: ColormapKind,
    },
    /// colorcet `__init__.py`: every `# cmap_def` table, named by its alias table.
    Colorcet {
        url: Remote,
    },
    /// cmocean `cm.py` `cmapnames`, each read from `<rgb_base><name>-rgb.txt`.
    Cmocean {
        cm_py: Remote,
        rgb_base: &'static str,
    },
    /// CMasher `.jscm` files listed through the GitHub tree API.
    Cmasher {
        tree_api: Remote,
        raw_base: &'static str,
    },
    /// Crameri's Zenodo archive: `<n>/<n>.txt` and `CategoricalPalettes/<n>S.txt`.
    CrameriZip {
        url: Remote,
        cache: &'static str,
    },
    /// cmyt modules `<base><name>.py`.
    Cmyt {
        base: &'static str,
        names: &'static [&'static str],
    },
    /// seaborn `SEABORN_PALETTES`.
    SeabornPalettes {
        url: Remote,
    },
    ColorBrewer {
        url: Remote,
    },
    Tol {
        url: Remote,
    },
    CartoTs {
        url: Remote,
    },
    /// An R `<list> <- list(name = c("#..."), ...)` of palettes.
    RList {
        url: Remote,
        list: &'static str,
    },
    /// YAML `Name:` / `- '#hex'` lists (ghibli).
    Yaml {
        url: Remote,
    },
    Catppuccin {
        url: Remote,
    },
    /// CSS custom properties `--<prefix>N` (Nord).
    CssVars {
        url: Remote,
        prefix: &'static str,
        name: &'static str,
    },
    /// Krzywinski palette file (`-main` rows).
    Krzywinski {
        url: Remote,
        name: &'static str,
    },
    /// Named schemes from a ColorSchemes.jl data file. Deliberate exception: only
    /// for palettes the ColorSchemes.jl authors created themselves (Paintings).
    ColorSchemesJl {
        url: Remote,
        names: &'static [&'static str],
    },
    /// Published values without a machine-readable file, cited in the notice.
    Hex {
        name: &'static str,
        kind: ColormapKind,
        colors: &'static [&'static str],
    },
}

pub struct Family {
    pub key: &'static str,
    pub name: &'static str,
    /// Display summary of the license(s).
    pub license_id: &'static str,
    /// Every license covering the family's maps, checked against [`COMPATIBLE_LICENSES`].
    pub spdx: &'static [&'static str],
    pub license: License,
    /// Attribution notice printed above the license text in `LICENSES.md`.
    pub notice: Option<&'static str>,
    pub source: &'static str,
    pub attribution: &'static str,
    pub sources: &'static [Source],
}

/// Every bundled family, in display order.
pub fn all() -> impl Iterator<Item = &'static Family> {
    classic::FAMILIES
        .iter()
        .chain(scientific::FAMILIES)
        .chain(palettes::FAMILIES)
}

pub use notices::MIT_TEMPLATE;

/// Rejects families covered by a license outside [`COMPATIBLE_LICENSES`].
pub fn check_compatible(family: &Family) -> Result<(), String> {
    match family
        .spdx
        .iter()
        .find(|id| !COMPATIBLE_LICENSES.contains(id))
    {
        Some(id) => Err(format!(
            "{}: license {id} is not compatible with MIT OR Apache-2.0",
            family.name
        )),
        None if family.spdx.is_empty() => Err(format!("{}: no license declared", family.name)),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_family_is_license_compatible() {
        for family in all() {
            assert_eq!(check_compatible(family), Ok(()));
        }
    }

    #[test]
    fn family_keys_are_unique() {
        let mut keys: Vec<&str> = all().map(|f| f.key).collect();
        let n = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), n);
        assert!(n < usize::from(u8::MAX));
    }

    #[test]
    fn copyleft_licenses_are_rejected() {
        let family = Family {
            key: "x",
            name: "X",
            license_id: "GPL-3.0-only",
            spdx: &["GPL-3.0-only"],
            license: License::Inline(""),
            notice: None,
            source: "",
            attribution: "",
            sources: &[],
        };
        assert!(check_compatible(&family).is_err());
    }
}
