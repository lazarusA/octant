//! Top-level application state, dimension configurations, and session management.

pub mod active_dataset;
pub mod alpha_state;
mod app_defaults;
pub mod app_state;
pub mod colormap_state;
pub mod dataset_activation;
pub mod dimension_state;
pub mod line_profile;
#[cfg(test)]
mod line_profile_tests;
pub mod line_series;
pub mod navigation;
mod notifications;
pub mod playback;
pub mod plot_configs;
pub mod plot_type;
pub mod session;
pub mod store_kind;
pub mod ui_layout;

pub use app_state::OctantApp;
pub use colormap_state::ColormapState;
pub use dimension_state::{AnimationRole, DimConfig, SpatialRole};
pub use navigation::NavigationState;
pub use playback::PlaybackState;
pub use plot_configs::PlotConfigs;
pub use store_kind::StoreKind;
pub use ui_layout::UiLayoutState;
