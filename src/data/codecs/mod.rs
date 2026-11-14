//! Codec pipeline normalization, plugins, and abstractions for Zarr datasets.

pub mod blusc_plugin;
pub mod capped;
#[cfg(test)]
mod capped_tests;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod grib_network_tests;
pub mod normalize;
#[cfg(test)]
mod pipeline_tests;
mod plugin;
mod scale_offset;
#[cfg(test)]
mod scale_offset_tests;
#[cfg(test)]
mod tests;
pub mod zstd_plugin;

pub use blusc_plugin::BluscCodec;
pub use normalize::{is_array_to_bytes_codec, normalize_v3_array_metadata};
pub use zstd_plugin::RuzstdCodec;
