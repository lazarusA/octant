//! Dataset gallery: the docs gallery's entries, fetched at startup
//! (`remote.rs`), then the built-in procedural ones (`procedural.rs`).

pub mod procedural;
pub mod remote;
pub mod types;

#[cfg(test)]
mod tests;

pub use procedural::procedural_entries;
pub use remote::{Gallery, current};
pub use types::{Domain, Format, GalleryEntry, shown};

/// Every entry: the gallery's (`gallery` as read this frame), then the
/// procedural ones.
pub fn all_entries(gallery: &Gallery) -> impl Iterator<Item = &GalleryEntry> {
    gallery.entries().iter().chain(procedural_entries())
}
