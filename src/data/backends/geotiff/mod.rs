//! GeoTIFF and TIFF storage backend for `BlockStore`.

pub mod blit;
pub mod coords;
pub mod decompress;
pub mod inspect;
pub mod palette;
pub mod predictor;
pub mod reader;
pub mod slice;
pub mod slice_striped;
pub mod slice_tiled;
pub mod store;
#[cfg(test)]
pub mod tests;
pub mod unpack;

pub use blit::{BlitSource, ReadWindow};
pub use coords::GeoSpatialBounds;
pub use inspect::inspect_tiff;
pub use reader::{MemoryTiffReader, create_async_reader};
pub use slice::fetch_geotiff_block;
pub use store::GeoTiffBlockStore;
