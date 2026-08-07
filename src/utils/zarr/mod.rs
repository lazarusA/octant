pub mod slice;
pub mod store;

pub use slice::{DimensionSelection, SliceRequest, fetch_slice, fetch_slice_range};

pub use store::build_sync_store;
