//! Dimension slider controls, range configuration, and slice calculations.

pub mod composite;
pub mod defaults;
pub mod double_slider;

pub mod metrics;
pub mod roles;
pub mod slice_req;
pub mod slider_row;

pub use crate::utils::format_byte_size;
pub use defaults::init_variable_dimension_defaults;
pub use double_slider::double_slider_with_inputs;
pub use metrics::{
    calculate_download_sizes, calculate_max_animated_steps, calculate_selected_2d_elements,
    calculate_selected_volume_elements, is_volume_allowed_for_selection,
};
pub use roles::apply_role_change;
pub use slice_req::{build_slice_request, build_slice_request_for_plotted};
pub use slider_row::show_dimension_sliders;
