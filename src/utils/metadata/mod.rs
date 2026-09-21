//! Dataset & Variable Metadata Discovery and CF conventions module.

pub mod array_open;
pub mod cf;
pub mod discovery;
pub mod node_info;
#[cfg(test)]
mod tests;

pub use array_open::{
    instantiate_array_from_node_metadata, normalize_v3_array_metadata,
    open_or_instantiate_array_normalized, resolve_array_candidate_store_keys,
    resolve_array_dimension_names, variable_info_from_array,
};
pub use cf::{
    ParsedCfAttributes, default_dimension_names_for_rank, find_first_attr, merge_parent_attributes,
    resolve_ancestor_attributes,
};
pub use discovery::{discover_arrays_via_http_metadata, extract_store_variables};
pub use node_info::{
    extract_group_attributes_from_node_metadata,
    extract_store_variables_from_consolidated_metadata, variable_info_from_node_metadata,
    variable_info_from_node_metadata_with_parent_attributes,
};
