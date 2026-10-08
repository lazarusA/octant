//! Plotted layers: what each one is drawn from (`Source`, `VariableSelection`),
//! its data, renderers, style and pending request, held in a `LayerStack` whose base layer is the
//! plot shown on the canvas.

mod data;
mod layer;
mod load;
mod renderers;
mod selection;
mod source;
mod stack;
mod style;
#[cfg(test)]
mod style_tests;

pub use data::LayerData;
pub use layer::Layer;
pub use load::LoadState;
pub use renderers::LayerRenderers;
pub use selection::VariableSelection;
pub use source::Source;
pub use stack::LayerStack;
pub use style::{ColorStyle, CompositeStyle};
