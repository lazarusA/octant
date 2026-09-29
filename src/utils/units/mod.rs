//! Dimensional units, CF temporal parsing, civil calendar, and coordinate formatting.

pub mod bytes;
pub mod calendar;
pub mod cf;
pub mod format;
#[cfg(test)]
mod tests;

pub use bytes::{
    calculate_variable_size_bytes, data_type_bytes, format_byte_size, format_count_metric,
};
pub use calendar::{add_days_to_date, civil_from_days, days_from_civil};
pub use cf::{
    is_cf_time_unit, parse_iso_date, parse_reference_date, parse_time_unit, split_since,
    unit_to_milliseconds,
};
pub use format::{
    format_axis_value, format_cardinal_degrees, format_coord_scalar, format_num_with_unit,
    format_scalar_coordinate, parse_loc,
};
