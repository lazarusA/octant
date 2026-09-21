//! Metadata representations for open dataset sources and variables.

pub mod dataset;
pub mod tree;
pub mod variable;

#[cfg(test)]
mod tests;

pub use dataset::DatasetMetadata;
pub use tree::VariableTreeGroup;
pub use variable::VariableInfo;
