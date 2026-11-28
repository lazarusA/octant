//! The built-in procedural datasets: synthetic, generated in the app, so
//! listed whether or not the gallery loaded.

use super::types::{Domain, Format, GalleryEntry};
use std::sync::LazyLock;

const ENTRIES: [(&str, &str, &str, &str); 4] = [
    (
        "procedural-healpix",
        "HEALPix Discrete Global Grid",
        "Synthetic Baroclinic Waves on Nside=16 & 32 HEALPix Discrete Global Grids",
        "procedural://healpix",
    ),
    (
        "procedural-irregular-grids",
        "Irregular Grids (Ground Truth)",
        "Clenshaw-Curtis, Gaussian, Geometrically Stretched & Stepped Resolution Grids",
        "procedural://irregular",
    ),
    (
        "procedural-4d-volume",
        "4D Known-Truth Gaussian Volume",
        "Synthetic 4D Traveling Wave Packet with Exact Ground Truth & Orbiting Center",
        "procedural://volume4d",
    ),
    (
        "procedural-2d-matrix",
        "2D Procedural Matrix",
        "Synthetic 2D Sine Wave Scalar Field for Quick Tests",
        "procedural://matrix2d",
    ),
];

/// The procedural entries, built once.
pub fn procedural_entries() -> &'static [GalleryEntry] {
    static BUILT: LazyLock<Vec<GalleryEntry>> = LazyLock::new(|| {
        ENTRIES
            .iter()
            .map(|&(key, label, subtitle, store)| GalleryEntry {
                key: key.into(),
                label: label.into(),
                subtitle: subtitle.into(),
                store: store.into(),
                format: Format::Procedural,
                domain: Domain::Procedural,
                ..Default::default()
            })
            .collect()
    });
    &BUILT
}
