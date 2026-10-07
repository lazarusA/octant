//! Metadata representations for open dataset sources and variables.

pub mod coord_values;
pub mod dataset;
pub mod tree;
pub mod variable;

#[cfg(test)]
mod coord_values_tests;
#[cfg(test)]
mod tests;

pub use coord_values::{CoordValue, CoordValues, SpacingCheck};
pub use dataset::DatasetMetadata;
pub use tree::VariableTreeGroup;
pub use variable::VariableInfo;
