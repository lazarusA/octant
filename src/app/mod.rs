//! `OctantApp`
//!
//! - `state`: struct definition, config enums (`StoreKind`, `SpatialRole`,
//!   `AnimationRole`, `DimConfig`), and construction.
//! - `actions`: `AppAction` + `dispatch`: event-driven state mutation.
//! - `data_loading`: store inspection, cache lookup/miss handling, and
//!   slice/variable loading (the I/O boundary).
//! - `pipeline`: GPU pipeline (re)build from `MatrixData`, color params,
//!   3D aspect ratio.
//! - `ui`: the `eframe::App` per-frame update/paint loop.

mod actions;
mod block_loading;
pub mod capture;
pub mod capture_ring;
mod data_loading;
mod pipeline;
mod state;
mod ui;

#[allow(unused_imports)]
pub use capture::{AspectRatioPreset, CaptureConfig};
pub use state::{AnimationRole, DimConfig, OctantApp, SpatialRole, StoreKind};
