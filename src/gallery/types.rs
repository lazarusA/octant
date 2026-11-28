//! Gallery entry types: the docs gallery's entries (`remote.rs`) and the
//! built-in procedural ones (`procedural.rs`).

use serde::Deserialize;
use std::collections::BTreeMap;

/// A store's format as the gallery names it. Only the badge and the format
/// chips read it: a store opens by its address, as a share link does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    Zarr,
    OmeZarr,
    Icechunk,
    #[serde(rename = "geotiff")]
    GeoTiff,
    #[serde(rename = "netcdf")]
    NetCdf,
    Hdf5,
    Grib,
    /// Octant's built-in synthetic datasets, never in the gallery.
    #[serde(skip_deserializing)]
    Procedural,
    /// A format a newer gallery lists.
    #[default]
    #[serde(other)]
    Other,
}

impl Format {
    /// Every format, in chip order.
    pub const ALL: [Format; 9] = [
        Format::Zarr,
        Format::OmeZarr,
        Format::Icechunk,
        Format::GeoTiff,
        Format::NetCdf,
        Format::Hdf5,
        Format::Grib,
        Format::Procedural,
        Format::Other,
    ];

    /// Its name, untranslated like every format name.
    pub fn name(self) -> &'static str {
        match self {
            Format::Zarr => "Zarr",
            Format::OmeZarr => "OME-Zarr",
            Format::Icechunk => "Icechunk",
            Format::GeoTiff => "GeoTIFF",
            Format::NetCdf => "NetCDF",
            Format::Hdf5 => "HDF5",
            Format::Grib => "GRIB",
            Format::Procedural => "Procedural",
            Format::Other => "Store",
        }
    }
}

/// What a dataset is about, as the gallery groups it (the domain chips).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Domain {
    Climate,
    Weather,
    Ocean,
    EarthObservation,
    Microscopy,
    Medical,
    Scivis,
    /// Octant's built-in synthetic datasets, never in the gallery.
    #[serde(skip_deserializing)]
    Procedural,
    /// Any other, or one a newer gallery adds.
    #[default]
    #[serde(other)]
    Other,
}

impl Domain {
    /// Every domain, in chip order.
    pub const ALL: [Domain; 9] = [
        Domain::Climate,
        Domain::Weather,
        Domain::Ocean,
        Domain::EarthObservation,
        Domain::Microscopy,
        Domain::Medical,
        Domain::Scivis,
        Domain::Other,
        Domain::Procedural,
    ];
}

/// One dataset of the gallery: an entry of the docs gallery's
/// `gallery.json` (fields it adds are ignored) or a procedural one.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct GalleryEntry {
    pub key: String,
    #[serde(rename = "title")]
    pub label: String,
    #[serde(rename = "description", default)]
    pub subtitle: String,
    pub store: String,
    pub format: Format,
    #[serde(default)]
    pub domain: Domain,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Who made the data, as they ask to be credited.
    #[serde(default)]
    pub credit: String,
    #[serde(default)]
    pub license: String,
    /// Where the license's terms are stated.
    pub license_url: Option<String>,
    pub citation: Option<String>,
    /// The provider's page for the dataset.
    pub source_page: Option<String>,
    /// Its picture's path on the docs site (`/gallery/<key>.webp`).
    pub thumbnail: Option<String>,
    /// Share-link keys applied once the store opens (`var`, `plot`, `cmap`...),
    /// the view the gallery shows.
    #[serde(default)]
    pub open: BTreeMap<String, String>,
    /// Its server sends no CORS headers: only the desktop app can read it.
    #[serde(default)]
    pub desktop_only: bool,
}

impl GalleryEntry {
    /// Its page on the docs gallery; none for a procedural entry.
    pub fn gallery_page(&self) -> Option<String> {
        (self.domain != Domain::Procedural)
            .then(|| format!("{}/gallery/{}", super::remote::SITE, self.key))
    }

    /// What opening it submits: its store, or a share link of the store and
    /// its view (`app/share/`).
    pub fn link(&self) -> String {
        if self.open.is_empty() {
            return self.store.trim().to_string();
        }
        let mut query = String::new();
        crate::app::share::pair(&mut query, "src", self.store.trim());
        for (key, value) in &self.open {
            crate::app::share::pair(&mut query, key, value);
        }
        format!("?{query}")
    }
}

/// Whether `entry` is shown under the chips picked (`None`: All).
pub fn shown(entry: &GalleryEntry, domain: Option<Domain>, format: Option<Format>) -> bool {
    domain.is_none_or(|d| entry.domain == d) && format.is_none_or(|f| entry.format == f)
}
