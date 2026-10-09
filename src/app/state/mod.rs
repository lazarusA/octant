//! Top-level application state, dimension configurations, and session management.

pub mod active_dataset;
pub mod alpha_state;
mod app_defaults;
pub mod app_state;
pub mod colormap_state;
pub mod dataset_activation;
pub mod dimension_state;
pub mod line_series;
mod notifications;
pub mod plot_type;
pub mod session;
pub mod store_kind;

pub use app_state::OctantApp;
pub use colormap_state::ColormapState;
pub use dimension_state::{AnimationRole, DimConfig, SpatialRole};
pub use store_kind::StoreKind;
