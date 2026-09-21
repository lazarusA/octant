//! Top-level application state, dimension configurations, multi-layer plotting, and session management.

pub mod app_state;
pub mod dataset_activation;
pub mod dimension_state;
pub mod layer_state;
pub mod session;
pub mod store_kind;

pub use app_state::OctantApp;
pub use dimension_state::{AnimationRole, DimConfig, SpatialRole};
#[allow(unused_imports)]
pub use layer_state::PlottedVariableState;
pub use store_kind::StoreKind;
