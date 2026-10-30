//! OME-Zarr (OME-NGFF) metadata, read with `ome_zarr_metadata`.
//!
//! - `parse.rs`: group attributes -> 0.5 fields, field by field, with
//!   `legacy.rs` (0.1-0.3 upgraded to 0.4) and `repair.rs` (common quirks).
//! - `model.rs`, `axes.rs`, `omero.rs`, `image.rs`: Octant's own view of an
//!   image (axes and units, levels with scale and translation, channels).
//! - `plate.rs`: plates, wells and what a store holds beyond its images.
//! - `reader.rs`, `discover.rs`, `direct.rs`, `variables.rs`: walking a store
//!   in passes, each pass's missing keys read concurrently, with `ome_xml.rs` naming
//!   `bioformats2raw` series.

pub mod axes;
pub mod coords;
pub mod direct;
mod discover;
mod image;
mod legacy;
pub mod level;
mod meta_cache;
pub mod model;
mod ome_xml;
mod omero;
pub mod parse;
pub mod plate;
pub mod reader;
mod repair;
mod series;
mod variables;

#[cfg(test)]
mod block_tests;
#[cfg(test)]
mod container_tests;
#[cfg(test)]
mod legacy_tests;
#[cfg(test)]
mod network_tests;
#[cfg(test)]
mod ome_xml_tests;
#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use zarrs::storage::ReadableStorageTraits;

pub use axes::AxisKind;
pub use discover::{Discovery, Outcome, discover};
pub use meta_cache::MetaCache;
pub use model::{ChannelFamily, ChannelWindow, OmeChannel, OmeLabelInfo, OmeRef};
pub use plate::{OmeDataset, OmePlate, OmeWell};
pub use reader::{MetaReader, Prefetch};

/// Walks a store that answers every key directly (desktop), through the
/// store's `cache`; `None` when its root is not an OME-Zarr group.
pub fn discover_direct(
    storage: Arc<dyn ReadableStorageTraits>,
    cache: &MetaCache,
) -> Option<Discovery> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        direct::discover_concurrent(storage, cache)
    }
    #[cfg(target_arch = "wasm32")]
    match discover(&MetaReader::direct(storage).with_cache(cache), "") {
        Outcome::Found(found) => Some(found),
        Outcome::NotOme | Outcome::Pending => None,
    }
}

/// Logs a discovery's warnings and reports one toast when the metadata did
/// not conform but could still be read.
pub fn report_warnings(found: &Discovery, dataset: &str) {
    if found.warnings.is_empty() {
        return;
    }
    for w in &found.warnings {
        log::warn!("OME-Zarr metadata of {dataset}: {w}");
    }
    let version = if found.dataset.version.is_empty() {
        "?"
    } else {
        found.dataset.version.as_str()
    };
    crate::ui::toast::report(
        crate::ui::toast::Severity::Warning,
        crate::i18n::tr(crate::i18n::NoticesKey::OmeMetadataNonConforming),
        crate::i18n::filled(
            crate::i18n::tr(crate::i18n::NoticesKey::OmeMetadataNonConformingDetail),
            &[&version, &dataset],
        ),
    );
}
