//! NetCDF backend implementation for `BlockStore`.
//!
//! Supports reading classic NetCDF, 64-bit offset, CDF-5, and NetCDF-4/HDF5 files.

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;

    use netcdf::types::{FloatType, IntType, NcVariableType};
    use netcdf::{AttributeValue, Extent, Extents};

    use crate::data::block_request::BlockResult;
    use crate::data::block_store::{BlockStore, BlockStoreError};
    use crate::data::metadata::{DatasetMetadata, VariableInfo};
    use crate::data::octant_block::OctantBlock;
    use crate::data::slice_request::{DimensionSelection, SliceRequest};
    use crate::utils::grid::check_and_orient_block_grid;

    /// NetCDF storage backend reading local files via `libnetcdf`.
    #[derive(Debug, Clone)]
    pub struct NetCdfBlockStore {
        file_path: String,
    }

    impl NetCdfBlockStore {
        /// Opens a NetCDF dataset from a local filesystem path.
        pub fn open_local(path: &str) -> Result<Self, BlockStoreError> {
            let p = Path::new(path);
            if !p.exists() {
                return Err(format!("NetCDF file not found: {path}").into());
            }

            // Verify that the file can be opened and parsed by netcdf
            let _file =
                netcdf::open(p).map_err(|e| format!("Failed to open NetCDF file '{path}': {e}"))?;

            Ok(Self {
                file_path: path.to_string(),
            })
        }

        pub fn file_path(&self) -> &str {
            &self.file_path
        }
    }

    /// Helper to convert a NetCDF `AttributeValue` into a displayable string.
    fn attribute_value_to_string(attr: &AttributeValue) -> String {
        match attr {
            AttributeValue::Str(s) => s.clone(),
            AttributeValue::Float(f) => format!("{f}"),
            AttributeValue::Double(d) => format!("{d}"),
            AttributeValue::Schar(i) => format!("{i}"),
            AttributeValue::Uchar(u) => format!("{u}"),
            AttributeValue::Short(i) => format!("{i}"),
            AttributeValue::Ushort(u) => format!("{u}"),
            AttributeValue::Int(i) => format!("{i}"),
            AttributeValue::Uint(u) => format!("{u}"),
            AttributeValue::Longlong(i) => format!("{i}"),
            AttributeValue::Ulonglong(u) => format!("{u}"),
            AttributeValue::Floats(v) => format!("{v:?}"),
            AttributeValue::Doubles(v) => format!("{v:?}"),
            AttributeValue::Schars(v) => format!("{v:?}"),
            AttributeValue::Uchars(v) => format!("{v:?}"),
            AttributeValue::Shorts(v) => format!("{v:?}"),
            AttributeValue::Ushorts(v) => format!("{v:?}"),
            AttributeValue::Ints(v) => format!("{v:?}"),
            AttributeValue::Uints(v) => format!("{v:?}"),
            AttributeValue::Longlongs(v) => format!("{v:?}"),
            AttributeValue::Ulonglongs(v) => format!("{v:?}"),
            AttributeValue::Strs(v) => v.join(", "),
        }
    }

    /// Helper to convert a NetCDF `AttributeValue` into an `f64` for scale/offset/fill calculations.
    fn attribute_value_to_f64(attr: &AttributeValue) -> Option<f64> {
        match attr {
            AttributeValue::Double(d) => Some(*d),
            AttributeValue::Float(f) => Some(*f as f64),
            AttributeValue::Int(i) => Some(*i as f64),
            AttributeValue::Uint(u) => Some(*u as f64),
            AttributeValue::Short(s) => Some(*s as f64),
            AttributeValue::Ushort(u) => Some(*u as f64),
            AttributeValue::Schar(i) => Some(*i as f64),
            AttributeValue::Uchar(u) => Some(*u as f64),
            AttributeValue::Longlong(l) => Some(*l as f64),
            AttributeValue::Ulonglong(u) => Some(*u as f64),
            AttributeValue::Doubles(v) => v.first().copied(),
            AttributeValue::Floats(v) => v.first().map(|&f| f as f64),
            AttributeValue::Ints(v) => v.first().map(|&i| i as f64),
            AttributeValue::Uints(v) => v.first().map(|&u| u as f64),
            AttributeValue::Shorts(v) => v.first().map(|&s| s as f64),
            AttributeValue::Ushorts(v) => v.first().map(|&u| u as f64),
            AttributeValue::Schars(v) => v.first().map(|&i| i as f64),
            AttributeValue::Uchars(v) => v.first().map(|&u| u as f64),
            AttributeValue::Longlongs(v) => v.first().map(|&l| l as f64),
            AttributeValue::Ulonglongs(v) => v.first().map(|&u| u as f64),
            AttributeValue::Str(s) => s.trim().parse::<f64>().ok(),
            AttributeValue::Strs(v) => v.first().and_then(|s| s.trim().parse::<f64>().ok()),
        }
    }

    /// Converts a NetCDF `NcVariableType` to a descriptive string for metadata.
    fn var_type_to_string(vartype: &NcVariableType) -> &'static str {
        match vartype {
            NcVariableType::Float(FloatType::F32) => "float32",
            NcVariableType::Float(FloatType::F64) => "float64",
            NcVariableType::Int(IntType::I32) => "int32",
            NcVariableType::Int(IntType::I16) => "int16",
            NcVariableType::Int(IntType::I8) => "int8",
            NcVariableType::Int(IntType::I64) => "int64",
            NcVariableType::Int(IntType::U32) => "uint32",
            NcVariableType::Int(IntType::U16) => "uint16",
            NcVariableType::Int(IntType::U8) => "uint8",
            NcVariableType::Int(IntType::U64) => "uint64",
            NcVariableType::Char => "char",
            NcVariableType::String => "string",
            NcVariableType::Compound(_) => "compound",
            NcVariableType::Opaque(_) => "opaque",
            NcVariableType::Enum(_) => "enum",
            NcVariableType::Vlen(_) => "vlen",
        }
    }

    /// Read raw numeric values from a NetCDF variable, convert to `f32`, and apply scale/offset/fill masking.
    fn read_variable_hyperslab_as_f32(
        var: &netcdf::Variable<'_>,
        extents: &Extents,
    ) -> Result<Vec<f32>, BlockStoreError> {
        let scale_factor = var
            .attribute_value("scale_factor")
            .and_then(|r| r.ok())
            .and_then(|a| attribute_value_to_f64(&a));
        let add_offset = var
            .attribute_value("add_offset")
            .and_then(|r| r.ok())
            .and_then(|a| attribute_value_to_f64(&a));
        let fill_value = var
            .attribute_value("_FillValue")
            .or_else(|| var.attribute_value("missing_value"))
            .and_then(|r| r.ok())
            .and_then(|a| attribute_value_to_f64(&a));

        let has_calibration = scale_factor.is_some() || add_offset.is_some();
        let scale = scale_factor.unwrap_or(1.0);
        let offset = add_offset.unwrap_or(0.0);

        let transform_f64 = |val: f64| -> f32 {
            if let Some(fv) = fill_value
                && ((val - fv).abs() < 1e-5 || val.is_nan())
            {
                return f32::NAN;
            }
            if has_calibration {
                (val * scale + offset) as f32
            } else {
                val as f32
            }
        };

        match var.vartype() {
            NcVariableType::Float(FloatType::F32) => {
                let raw_vals: Vec<f32> = var
                    .get_values::<f32, _>(extents)
                    .map_err(|e| format!("Failed reading float32 hyperslab: {e}"))?;

                if !has_calibration && fill_value.is_none() {
                    Ok(raw_vals)
                } else {
                    Ok(raw_vals
                        .into_iter()
                        .map(|v| transform_f64(v as f64))
                        .collect())
                }
            }
            NcVariableType::Float(FloatType::F64) => {
                let raw_vals: Vec<f64> = var
                    .get_values::<f64, _>(extents)
                    .map_err(|e| format!("Failed reading float64 hyperslab: {e}"))?;

                Ok(raw_vals.into_iter().map(transform_f64).collect())
            }
            NcVariableType::Int(IntType::I32) => {
                let raw_vals: Vec<i32> = var
                    .get_values::<i32, _>(extents)
                    .map_err(|e| format!("Failed reading int32 hyperslab: {e}"))?;

                Ok(raw_vals
                    .into_iter()
                    .map(|v| transform_f64(v as f64))
                    .collect())
            }
            NcVariableType::Int(IntType::I16) => {
                let raw_vals: Vec<i16> = var
                    .get_values::<i16, _>(extents)
                    .map_err(|e| format!("Failed reading int16 hyperslab: {e}"))?;

                Ok(raw_vals
                    .into_iter()
                    .map(|v| transform_f64(v as f64))
                    .collect())
            }
            NcVariableType::Int(IntType::I8) => {
                let raw_vals: Vec<i8> = var
                    .get_values::<i8, _>(extents)
                    .map_err(|e| format!("Failed reading int8 hyperslab: {e}"))?;

                Ok(raw_vals
                    .into_iter()
                    .map(|v| transform_f64(v as f64))
                    .collect())
            }
            NcVariableType::Int(IntType::U32) => {
                let raw_vals: Vec<u32> = var
                    .get_values::<u32, _>(extents)
                    .map_err(|e| format!("Failed reading uint32 hyperslab: {e}"))?;

                Ok(raw_vals
                    .into_iter()
                    .map(|v| transform_f64(v as f64))
                    .collect())
            }
            NcVariableType::Int(IntType::U16) => {
                let raw_vals: Vec<u16> = var
                    .get_values::<u16, _>(extents)
                    .map_err(|e| format!("Failed reading uint16 hyperslab: {e}"))?;

                Ok(raw_vals
                    .into_iter()
                    .map(|v| transform_f64(v as f64))
                    .collect())
            }
            NcVariableType::Int(IntType::U8) | NcVariableType::Char => {
                let raw_vals: Vec<u8> = var
                    .get_values::<u8, _>(extents)
                    .map_err(|e| format!("Failed reading uint8 hyperslab: {e}"))?;

                Ok(raw_vals
                    .into_iter()
                    .map(|v| transform_f64(v as f64))
                    .collect())
            }
            NcVariableType::Int(IntType::I64) => {
                let raw_vals: Vec<i64> = var
                    .get_values::<i64, _>(extents)
                    .map_err(|e| format!("Failed reading int64 hyperslab: {e}"))?;

                Ok(raw_vals
                    .into_iter()
                    .map(|v| transform_f64(v as f64))
                    .collect())
            }
            NcVariableType::Int(IntType::U64) => {
                let raw_vals: Vec<u64> = var
                    .get_values::<u64, _>(extents)
                    .map_err(|e| format!("Failed reading uint64 hyperslab: {e}"))?;

                Ok(raw_vals
                    .into_iter()
                    .map(|v| transform_f64(v as f64))
                    .collect())
            }
            other => Err(format!("Unsupported NetCDF variable type: {other:?}").into()),
        }
    }

    /// Extracts 1D coordinate vectors or bounds from a NetCDF file.
    fn extract_dimension_coordinates(file: &netcdf::File) -> HashMap<String, Vec<String>> {
        let mut dimension_coordinates = HashMap::new();

        for var in file.variables() {
            let dims = var.dimensions();
            let name = var.name();
            let clean = name.trim().to_lowercase();

            // Coordinate variables in CF conventions are 1D variables with the same name as their dimension,
            // or explicit spatial coordinates like lat, lon, latitude, longitude, time, lev, level, depth.
            if dims.len() == 1 {
                let dim_len = dims[0].len();
                if dim_len == 0 {
                    continue;
                }

                let is_coord_var = dims[0].name() == name
                    || crate::utils::coordinates::is_spatial_x_name(&clean)
                    || crate::utils::coordinates::is_spatial_y_name(&clean)
                    || crate::utils::coordinates::is_spatial_z_name(&clean)
                    || clean == "time"
                    || clean == "depth"
                    || clean == "lev"
                    || clean == "level";

                if is_coord_var {
                    let extents = Extents::from(vec![Extent::SliceCount {
                        start: 0,
                        count: dim_len,
                        stride: 1,
                    }]);

                    if let Ok(values) = read_variable_hyperslab_as_f32(&var, &extents) {
                        let strings: Vec<String> =
                            values.into_iter().map(|v| format!("{v}")).collect();

                        dimension_coordinates.insert(clean.clone(), strings.clone());
                        if clean != name {
                            dimension_coordinates.insert(name.clone(), strings);
                        }
                    }
                }
            }
        }

        dimension_coordinates
    }

    impl BlockStore for NetCdfBlockStore {
        fn backend_name(&self) -> &str {
            "NetCDF"
        }

        fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
            let file = netcdf::open(&self.file_path)
                .map_err(|e| format!("Failed to open NetCDF file '{}': {e}", self.file_path))?;

            Ok(file.variables().map(|v| v.name()).collect())
        }

        fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
            let file = netcdf::open(&self.file_path)
                .map_err(|e| format!("Failed to open NetCDF file '{}': {e}", self.file_path))?;

            let file_size = std::fs::metadata(&self.file_path)
                .map(|m| m.len())
                .unwrap_or(0);

            let mut variables = Vec::new();

            for var in file.variables() {
                let name = var.name();
                let vartype = var.vartype();
                let data_type = var_type_to_string(&vartype).to_string();

                let dims = var.dimensions();
                let shape: Vec<u64> = dims.iter().map(|d| d.len() as u64).collect();
                let dimension_names: Vec<String> = dims.iter().map(|d| d.name()).collect();
                let chunk_shape = shape.clone();

                let mut attributes = HashMap::new();
                for attr in var.attributes() {
                    let attr_name = attr.name().to_string();
                    if let Ok(val) = attr.value() {
                        attributes.insert(attr_name, attribute_value_to_string(&val));
                    }
                }

                let units = attributes.get("units").cloned();
                let long_name = attributes
                    .get("long_name")
                    .cloned()
                    .or_else(|| attributes.get("standard_name").cloned());

                let time_coverage_start = attributes.get("time_coverage_start").cloned();
                let time_coverage_end = attributes.get("time_coverage_end").cloned();
                let temporal_resolution = attributes.get("temporal_resolution").cloned();

                variables.push(VariableInfo {
                    name,
                    data_type,
                    shape,
                    dimension_names,
                    chunk_shape,
                    file_size,
                    units,
                    long_name,
                    time_coverage_start,
                    time_coverage_end,
                    temporal_resolution,
                    attributes,
                });
            }

            let dimension_coordinates = extract_dimension_coordinates(&file);

            let dataset_name = Path::new(&self.file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("NetCDF Dataset")
                .to_string();

            Ok(DatasetMetadata {
                name: dataset_name,
                store_type: "NetCDF".to_string(),
                variables,
                dimension_coordinates,
            })
        }

        fn fetch_block_with_progress(
            &self,
            request: &SliceRequest,
            mut on_progress: Option<&mut (dyn FnMut(u64) + Send)>,
        ) -> Result<OctantBlock, BlockStoreError> {
            let file = netcdf::open(&self.file_path)
                .map_err(|e| format!("Failed to open NetCDF file '{}': {e}", self.file_path))?;

            let var = file.variable(&request.variable).ok_or_else(|| {
                format!("Variable '{}' not found in NetCDF file", request.variable)
            })?;

            let dims = var.dimensions();
            let rank = dims.len();

            if request.selections.len() != rank {
                return Err(format!(
                    "fetch_block: selection has {} dimension(s) but '{}' has rank {}",
                    request.selections.len(),
                    request.variable,
                    rank
                )
                .into());
            }

            let mut extents_vec = Vec::with_capacity(rank);
            let mut block_shape = Vec::with_capacity(rank);
            let mut origin = Vec::with_capacity(rank);
            let mut dim_names = Vec::with_capacity(rank);

            for (i, sel) in request.selections.iter().enumerate() {
                let dim_len = dims[i].len();
                let dim_name = dims[i].name();
                dim_names.push(dim_name);

                let (start, end) = match sel {
                    DimensionSelection::Index(idx) => (*idx, idx.saturating_add(1)),
                    DimensionSelection::Range { start, end } => (*start, *end),
                };

                let start = start.min(dim_len.saturating_sub(1));
                let end = end.max(start + 1).min(dim_len);
                let count = end - start;

                extents_vec.push(Extent::SliceCount {
                    start,
                    count,
                    stride: 1,
                });
                block_shape.push(count);
                origin.push(start);
            }

            let extents = Extents::from(extents_vec);
            let raw_values = read_variable_hyperslab_as_f32(&var, &extents)?;

            let bytes_read = raw_values
                .len()
                .checked_mul(std::mem::size_of::<f32>())
                .unwrap_or(0) as u64;

            if let Some(ref mut cb) = on_progress {
                cb(bytes_read);
            }

            let mut attributes = HashMap::new();
            for attr in var.attributes() {
                let attr_name = attr.name().to_string();
                if let Ok(val) = attr.value() {
                    attributes.insert(attr_name, attribute_value_to_string(&val));
                }
            }

            // Extract coordinates for the sliced block
            let mut coordinates: HashMap<String, Vec<f64>> = HashMap::new();
            for (i, name) in dim_names.iter().enumerate() {
                let clean = name.trim().to_lowercase();
                if let Some(coord_var) = file.variable(name).or_else(|| file.variable(&clean))
                    && coord_var.dimensions().len() == 1
                {
                    let dim_len = coord_var.dimensions()[0].len();
                    let (start, end) = (origin[i], origin[i] + block_shape[i]);
                    let start = start.min(dim_len.saturating_sub(1));
                    let end = end.min(dim_len);
                    let count = end.saturating_sub(start).max(1);

                    let coord_extents = Extents::from(vec![Extent::SliceCount {
                        start,
                        count,
                        stride: 1,
                    }]);

                    if let Ok(vals) = read_variable_hyperslab_as_f32(&coord_var, &coord_extents)
                        && let (Some(&first), Some(&last)) = (vals.first(), vals.last())
                    {
                        coordinates.insert(name.clone(), vec![first as f64, last as f64]);
                        if clean != *name {
                            coordinates.insert(clean, vec![first as f64, last as f64]);
                        }
                    }
                }
            }

            let json_attrs: serde_json::Map<String, serde_json::Value> = attributes
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();

            let oriented_values = check_and_orient_block_grid(
                raw_values,
                &mut block_shape,
                &mut dim_names,
                &mut origin,
                &json_attrs,
                &coordinates,
            );

            let block = OctantBlock::new(
                request.variable.clone(),
                block_shape,
                dim_names,
                origin,
                Arc::from(oriented_values.into_boxed_slice()),
                coordinates,
                attributes,
            );

            Ok(block)
        }

        fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
            use rayon::prelude::*;

            let blocks: Result<Vec<OctantBlock>, BlockStoreError> = requests
                .par_iter()
                .map(|request| self.fetch_block(request))
                .collect();

            Ok(BlockResult::new(blocks?))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::NetCdfBlockStore;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::data::block_request::BlockResult;
    use crate::data::block_store::{BlockStore, BlockStoreError};
    use crate::data::metadata::DatasetMetadata;
    use crate::data::octant_block::OctantBlock;
    use crate::data::slice_request::SliceRequest;

    #[derive(Debug, Clone)]
    pub struct NetCdfBlockStore;

    impl NetCdfBlockStore {
        pub fn open_local(_path: &str) -> Result<Self, BlockStoreError> {
            Err("NetCDF storage backend is not supported on WebAssembly targets".into())
        }
    }

    impl BlockStore for NetCdfBlockStore {
        fn backend_name(&self) -> &str {
            "NetCDF (unsupported on WASM)"
        }

        fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
            Err("NetCDF is not supported on WASM".into())
        }

        fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
            Err("NetCDF is not supported on WASM".into())
        }

        fn fetch_block_with_progress(
            &self,
            _request: &SliceRequest,
            _on_progress: Option<&mut (dyn FnMut(u64) + Send)>,
        ) -> Result<OctantBlock, BlockStoreError> {
            Err("NetCDF is not supported on WASM".into())
        }

        fn fetch_blocks(&self, _requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
            Err("NetCDF is not supported on WASM".into())
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::NetCdfBlockStore;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use crate::data::block_store::BlockStore;
    use crate::data::slice_request::SliceRequest;

    #[test]
    fn test_create_and_read_netcdf() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("octant_test_data.nc");
        let path_str = test_file.to_str().unwrap().to_string();

        // Create a test NetCDF file
        {
            let mut file = netcdf::create(&test_file).expect("create netcdf file");
            file.add_dimension("lat", 4).expect("add lat dim");
            file.add_dimension("lon", 5).expect("add lon dim");

            let mut lat_var = file
                .add_variable::<f32>("lat", &["lat"])
                .expect("add lat var");
            lat_var
                .put_values(&[-90.0f32, -30.0, 30.0, 90.0], ..)
                .expect("put lat values");

            let mut lon_var = file
                .add_variable::<f32>("lon", &["lon"])
                .expect("add lon var");
            lon_var
                .put_values(&[-180.0f32, -90.0, 0.0, 90.0, 180.0], ..)
                .expect("put lon values");

            let mut temp_var = file
                .add_variable::<f32>("temperature", &["lat", "lon"])
                .expect("add temperature var");
            temp_var
                .put_attribute("units", "degC")
                .expect("put units attr");
            temp_var
                .put_attribute("long_name", "Surface Temperature")
                .expect("put long_name attr");

            let data: Vec<f32> = (0..20).map(|i| i as f32 * 1.5).collect();
            temp_var.put_values(&data, ..).expect("put temp data");
        }

        // Open with NetCdfBlockStore
        let store = NetCdfBlockStore::open_local(&path_str).expect("open store");
        let vars = store.variables().expect("get variables");
        assert!(vars.contains(&"temperature".to_string()));
        assert!(vars.contains(&"lat".to_string()));
        assert!(vars.contains(&"lon".to_string()));

        // Inspect metadata
        let metadata = store.inspect().expect("inspect metadata");
        assert_eq!(metadata.store_type, "NetCDF");
        let temp_info = metadata
            .variables
            .iter()
            .find(|v| v.name == "temperature")
            .expect("find temperature info");
        assert_eq!(temp_info.shape, vec![4, 5]);
        assert_eq!(temp_info.dimension_names, vec!["lat", "lon"]);
        assert_eq!(temp_info.units.as_deref(), Some("degC"));
        assert_eq!(temp_info.long_name.as_deref(), Some("Surface Temperature"));

        // Fetch full 2D block (Y oriented from North to South for GPU rendering)
        let req = SliceRequest::full_range("temperature", &[4, 5]);
        let block = store.fetch_block(&req).expect("fetch full block");
        assert_eq!(block.shape, vec![4, 5]);
        assert_eq!(block.values.len(), 20);
        // Row 0 of oriented block corresponds to North (+90 lat, row index 3 of raw data: 15..20)
        assert_eq!(block.values[0], 22.5);
        // Row 3 of oriented block corresponds to South (-90 lat, row index 0 of raw data: 0..5)
        assert_eq!(block.values[15], 0.0);

        // Fetch 2D sub-slice
        let sub_req = SliceRequest::new(
            "temperature",
            vec![
                crate::data::slice_request::DimensionSelection::range(1, 3),
                crate::data::slice_request::DimensionSelection::range(2, 5),
            ],
        );
        let sub_block = store.fetch_block(&sub_req).expect("fetch sub block");
        assert_eq!(sub_block.shape, vec![2, 3]);
        assert_eq!(sub_block.values.len(), 6);

        // Clean up
        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_netcdf_scale_offset_and_fill() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("octant_test_scale_fill.nc");
        let path_str = test_file.to_str().unwrap().to_string();

        {
            let mut file = netcdf::create(&test_file).expect("create netcdf file");
            file.add_dimension("dim0", 2).expect("add dim0");
            file.add_dimension("dim1", 2).expect("add dim1");

            let mut var = file
                .add_variable::<i16>("calibrated_data", &["dim0", "dim1"])
                .expect("add var");
            var.put_attribute("scale_factor", 0.1f32)
                .expect("scale attr");
            var.put_attribute("add_offset", 10.0f32)
                .expect("offset attr");
            var.put_attribute("_FillValue", -999i16).expect("fill attr");

            // [100, -999, 200, 0] -> expected [100 * 0.1 + 10 = 20.0, NaN, 200 * 0.1 + 10 = 30.0, 0 * 0.1 + 10 = 10.0]
            var.put_values(&[100i16, -999, 200, 0], ..)
                .expect("put values");
        }

        let store = NetCdfBlockStore::open_local(&path_str).expect("open store");
        let req = SliceRequest::full_range("calibrated_data", &[2, 2]);
        let block = store.fetch_block(&req).expect("fetch block");

        assert_eq!(block.values.len(), 4);
        assert!((block.values[0] - 20.0).abs() < 1e-4);
        assert!(block.values[1].is_nan());
        assert!((block.values[2] - 30.0).abs() < 1e-4);
        assert!((block.values[3] - 10.0).abs() < 1e-4);

        let _ = std::fs::remove_file(test_file);
    }
}
