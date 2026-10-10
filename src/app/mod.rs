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
//! - `floating`: the floating panels and colorbars over the canvas.
//! - `ui`: the `eframe::App` per-frame update/paint loop.

mod actions;
mod block_loading;
pub mod canvas;
pub mod controllers;
mod data_loading;
mod export_lifecycle;
mod floating;
#[cfg(test)]
mod floating_tests;
pub mod layers;
#[cfg(test)]
mod overlay_composite_tests;
#[cfg(test)]
mod overlay_state_tests;
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
pub use state::line_profile::LineLayout;
pub use state::line_series::{LineColoring, series_line_at, series_t};
pub use state::{AnimationRole, DimConfig, OctantApp, SpatialRole, StoreKind};
