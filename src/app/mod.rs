//! `OctantApp`
//!
//! - `state`: struct definition, config enums (`StoreKind`, `SpatialRole`,
//!   `AnimationRole`, `DimConfig`), and construction.
//! - `actions`: `AppAction` + `dispatch`: event-driven state mutation.
//! - `data_loading`: store inspection, cache lookup/miss handling, and
//!   slice/variable loading (the I/O boundary).
//! - `layers`: what a plot is drawn from (`VariableSelection`).
//! - `overlays`: adding, removing and aligning overlays over the base layer.
//! - `pipeline`: GPU pipeline (re)build from `MatrixData`, color params,
//!   3D aspect ratio.
//! - `ui`: the `eframe::App` per-frame update/paint loop.

mod actions;
mod block_loading;
mod data_loading;
pub mod layers;
pub mod overlays;
#[cfg(test)]
mod overlays_tests;
mod pipeline;
mod state;
#[cfg(test)]
pub(crate) mod test_support;
mod ui;

#[allow(unused_imports)] // used by tests and the library, not the binary
pub use layers::VariableSelection;
pub use state::{AnimationRole, DimConfig, OctantApp, SpatialRole, StoreKind};
