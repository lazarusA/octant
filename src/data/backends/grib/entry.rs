//! The variables a GRIB file shows and where each one's values come from.

use std::collections::HashMap;
use std::sync::Arc;

use crate::data::backends::netcdf::reader::NcVar;

/// Attributes from `(name, value)` pairs.
pub fn attrs(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    (pairs.iter())
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

/// Where a variable's values come from.
pub enum Item {
    /// One message per index of the leading dimensions (`None`: no message there, read as
    /// NaN), each a `ny` by `nx` plane: its offset and length in the file.
    Data {
        messages: Vec<Option<(u64, u64)>>,
        ny: usize,
        nx: usize,
    },
    /// A coordinate, computed while scanning.
    Values(Arc<[f64]>),
    /// A grid mapping: its attributes are what it says.
    Scalar,
}

/// One variable and its values.
pub struct Entry {
    pub var: NcVar,
    pub item: Item,
}

impl Entry {
    /// The 1D coordinate `name` along its own dimension.
    pub fn coordinate(name: &str, values: Arc<[f64]>, attrs: HashMap<String, String>) -> Self {
        let var = NcVar {
            path: name.to_owned(),
            data_type: "float64",
            dims: vec![(name.to_owned(), values.len())],
            attributes: Arc::new(attrs),
            calibration: Default::default(),
        };
        let item = Item::Values(values);
        Self { var, item }
    }

    /// The scalar grid mapping `name`.
    pub fn mapping(name: &str, attrs: HashMap<String, String>) -> Self {
        let var = NcVar {
            path: name.to_owned(),
            data_type: "int32",
            dims: Vec::new(),
            attributes: Arc::new(attrs),
            calibration: Default::default(),
        };
        let item = Item::Scalar;
        Self { var, item }
    }
}
