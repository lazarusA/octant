//! Plotted layers: what each one is drawn from (`Source`, `VariableSelection`),
//! its data, renderers, style and pending request, held in a `LayerStack` whose base layer is the
//! plot shown on the canvas.

mod alignment;
#[cfg(test)]
mod alignment_tests;
mod colorbar;
mod data;
mod id;
mod layer;
mod load;
mod renderers;
mod selection;
mod source;
mod stack;
#[cfg(test)]
mod stack_tests;
mod style;
#[cfg(test)]
mod style_tests;
mod window;

pub use alignment::{Alignment, classify};
pub use colorbar::{BarOrientation, ColorbarPlacement, Slot};
pub use data::LayerData;
pub use id::LayerId;
pub use layer::{LABEL_BUF, Layer};
pub use load::LoadState;
pub use renderers::LayerRenderers;
pub use selection::VariableSelection;
pub use source::Source;
pub use stack::LayerStack;
pub use style::{AlphaCurveState, ColorStyle, CompositeStyle};
pub use window::{follow_base_window, overlay_selection};
