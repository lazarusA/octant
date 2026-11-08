//! What the NetCDF backend reads of a file, whatever reads it: netCDF-C on the
//! desktop (`c_reader.rs`), `hdf5-pure` in the browser's data workers and for
//! MATLAB files (`pure/`). Inspection, coordinates, calibration, slicing and
//! orientation run once over it (`inspect.rs`, `coord_scan.rs`, `coords.rs`,
//! `slice.rs`), so a file shows the same on both.

use std::collections::HashMap;

use crate::data::DataCalibration;
use crate::data::blocks::BlockStoreError;

/// One variable as its reader describes it.
#[derive(Debug, Clone)]
pub struct NcVar {
    /// Its path from the root: `t`, or `group/t`.
    pub path: String,
    /// `float32`, `int16`, ..., `char` (`NC_CHAR`), `string`, or the kind of an
    /// unplottable type (`compound`, `opaque`, `enum`, `vlen`).
    pub data_type: &'static str,
    /// Each dimension's name and length, in storage order.
    pub dims: Vec<(String, usize)>,
    /// Its attributes as text, bookkeeping ones left out.
    pub attributes: HashMap<String, String>,
    /// CF scale, offset and masking, from its numeric attributes.
    pub calibration: DataCalibration,
}

impl NcVar {
    /// Its name within its group.
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    pub fn shape(&self) -> impl Iterator<Item = usize> + '_ {
        self.dims.iter().map(|(_, len)| *len)
    }
}

/// Values as read: `float32` variables as `f32`, every other number as `f64`.
pub enum Raw {
    F32(Vec<f32>),
    F64(Vec<f64>),
}

pub trait NcReader {
    /// The root group's attributes.
    fn global_attributes(&self) -> HashMap<String, String>;
    /// Every variable: the root's, then each group's followed by its subgroups'.
    fn variables(&self) -> Vec<NcVar>;
    /// The variable at `path`, or `None`.
    fn variable(&self, path: &str) -> Option<NcVar>;
    /// The window `start..start + count` of numeric `var`, uncalibrated.
    fn read(&self, var: &NcVar, start: &[usize], count: &[usize]) -> Result<Raw, BlockStoreError>;
    /// The labels of a text variable `coord_read::text_dimension` accepts.
    fn labels(&self, var: &NcVar) -> Option<Vec<String>>;
}
