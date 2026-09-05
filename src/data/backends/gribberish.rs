//! Gribberish backend implementation for `BlockStore`.
//!
//! Supports reading GRIB1 and GRIB2 files directly via the `gribberish` crate.

use std::collections::HashMap;

use crate::app::StoreKind;
use crate::data::block_request::BlockResult;
use crate::data::block_store::{BlockStore, BlockStoreError};
use crate::data::metadata::{DatasetMetadata, VariableInfo};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::utils::grid::check_and_orient_block_grid;
use crate::utils::path::{expand_tilde, infer_store_kind_from_target};
use crate::utils::units::calculate_variable_size_bytes;

/// Stores index metadata for a single GRIB message within a file.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GribMessageInfo {
    byte_offset: usize,
    message_len: usize,
    time_index: usize,
    time_str: String,
    level_index: usize,
    level_value: Option<f64>,
    grid_shape: (usize, usize), // (rows/nj/y, cols/ni/x)
}

/// Stores raw parsed information from scanning a GRIB message header.
#[derive(Debug, Clone)]
struct RawGribMessage {
    var_name: String,
    full_name: Option<String>,
    unit: Option<String>,
    time_str: String,
    level_val: Option<f64>,
    grid_shape: (usize, usize),
    byte_offset: usize,
    msg_len: usize,
}

/// Stores variable definition and its constituent GRIB messages.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GribVariable {
    name: String,
    dim_names: Vec<String>,
    shape: Vec<usize>,
    units: Option<String>,
    long_name: Option<String>,
    attributes: HashMap<String, String>,
    time_coords: Vec<String>,
    level_coords: Vec<String>,
    messages: Vec<GribMessageInfo>,
}

/// GRIB/GRIB2 storage backend reading local files via `gribberish`.
#[derive(Debug, Clone)]
pub struct GribberishBlockStore {
    file_path: String,
    metadata: DatasetMetadata,
    variables: HashMap<String, GribVariable>,
    dimension_coordinates: HashMap<String, Vec<String>>,
}

type GribIndexResult = (
    HashMap<String, GribVariable>,
    HashMap<String, Vec<String>>,
    DatasetMetadata,
);

impl GribberishBlockStore {
    /// Opens a GRIB or GRIB2 dataset from a local filesystem path.
    pub fn open_local(path: &str) -> Result<Self, BlockStoreError> {
        let clean_path = path
            .strip_prefix("file://")
            .or_else(|| path.strip_prefix("grib://"))
            .unwrap_or(path)
            .trim()
            .trim_matches('\'')
            .trim_matches('"');

        // Check inferred store kind to emit informative warnings if unusual or mismatched
        match infer_store_kind_from_target(clean_path) {
            Ok(kind) if kind != StoreKind::LocalGrib => {
                log::warn!(
                    "Gribberish backend: Target path '{clean_path}' was inferred as {kind:?}; attempting GRIB indexing regardless"
                );
            }
            Err(err) => {
                log::warn!(
                    "Gribberish backend: Target path '{clean_path}' could not be inferred as a known format ({err}); attempting GRIB indexing"
                );
            }
            _ => {}
        }

        let p = expand_tilde(clean_path);
        if !p.exists() {
            return Err(format!("File not found: {}", p.display()).into());
        }
        if p.is_dir() {
            return Err(format!(
                "'{}' is a directory. Please specify a .grib / .grib2 / .grb / .grb2 file path.",
                p.display()
            )
            .into());
        }

        let file_bytes = std::fs::read(&p)
            .map_err(|e| format!("Failed to read GRIB file '{}': {e}", p.display()))?;

        let dataset_name = p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "GRIB Dataset".to_string());

        let (variables, dimension_coordinates, metadata) =
            Self::index_grib_file(&file_bytes, &dataset_name)?;

        log::info!(
            "Successfully opened GRIB dataset '{}' with {} variables",
            p.display(),
            variables.len()
        );

        Ok(Self {
            file_path: p.to_string_lossy().to_string(),
            metadata,
            variables,
            dimension_coordinates,
        })
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Indexes all messages in the GRIB file and structures variables and coordinate axes.
    fn index_grib_file(
        data: &[u8],
        dataset_name: &str,
    ) -> Result<GribIndexResult, BlockStoreError> {
        let raw_messages = Self::extract_raw_messages(data)?;
        Self::build_indexed_dataset(raw_messages, dataset_name)
    }

    /// Scans byte stream and extracts all valid GRIB messages.
    fn extract_raw_messages(data: &[u8]) -> Result<Vec<RawGribMessage>, BlockStoreError> {
        let mut raw_messages = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            if let Some(msg) = gribberish::message::read_message(data, offset) {
                let msg_len = msg.len();
                let var_name = match msg.variable_abbrev().or_else(|_| msg.variable_name()) {
                    Ok(name) if !name.trim().is_empty() => name,
                    _ => {
                        let fallback = format!("var_{}", raw_messages.len());
                        log::warn!(
                            "GRIB message at byte offset {offset}: variable name could not be identified; defaulting to '{fallback}'"
                        );
                        fallback
                    }
                };

                let full_name = msg.variable_name().ok();
                let unit = msg.unit().ok();

                let time_str = match msg.forecast_date().or_else(|_| msg.reference_date()) {
                    Ok(d) => d.to_string(),
                    Err(_) => {
                        log::debug!(
                            "GRIB message for variable '{var_name}' at byte offset {offset}: missing valid forecast/reference date; defaulting time to '0'"
                        );
                        "0".to_string()
                    }
                };

                let level_val = msg.first_fixed_surface().ok().and_then(|(_, val)| val);
                let grid_shape = msg.grid_dimensions().unwrap_or((1, 1));

                raw_messages.push(RawGribMessage {
                    var_name,
                    full_name,
                    unit,
                    time_str,
                    level_val,
                    grid_shape,
                    byte_offset: offset,
                    msg_len,
                });

                offset = offset.saturating_add(msg_len.max(1));
            } else {
                offset = offset.saturating_add(1);
            }
        }

        if raw_messages.is_empty() {
            log::warn!(
                "No valid GRIB messages found in file buffer (length {} bytes)",
                data.len()
            );
            return Err("No valid GRIB messages found in file".into());
        }

        Ok(raw_messages)
    }

    /// Groups raw messages by variable name and builds dataset metadata and coordinate maps.
    fn build_indexed_dataset(
        raw_messages: Vec<RawGribMessage>,
        dataset_name: &str,
    ) -> Result<GribIndexResult, BlockStoreError> {
        let mut dim_coords: HashMap<String, Vec<String>> = HashMap::new();
        let mut var_groups: HashMap<String, Vec<RawGribMessage>> = HashMap::new();

        for item in raw_messages {
            var_groups
                .entry(item.var_name.clone())
                .or_default()
                .push(item);
        }

        let mut variables: HashMap<String, GribVariable> = HashMap::new();
        let mut var_info_list = Vec::new();

        for (var_name, msgs) in var_groups {
            // Collect unique times and levels
            let mut unique_times: Vec<String> = Vec::new();
            let mut unique_levels: Vec<Option<f64>> = Vec::new();

            for m in &msgs {
                if !unique_times.contains(&m.time_str) {
                    unique_times.push(m.time_str.clone());
                }
                if !unique_levels.contains(&m.level_val) {
                    unique_levels.push(m.level_val);
                }
            }

            let num_times = unique_times.len();
            let num_levels = unique_levels.len();
            let (nj, ni) = msgs.first().map(|m| m.grid_shape).unwrap_or((1, 1));

            // Default coordinate fallback for lat/lon
            dim_coords
                .entry("latitude".to_string())
                .or_insert_with(|| (0..nj).map(|y| y.to_string()).collect());
            dim_coords
                .entry("longitude".to_string())
                .or_insert_with(|| (0..ni).map(|x| x.to_string()).collect());

            let mut dim_names = Vec::new();
            let mut shape = Vec::new();

            if num_times > 1 || (num_times == 1 && num_levels > 1) {
                dim_names.push("time".to_string());
                shape.push(num_times);
            }
            if num_levels > 1 {
                dim_names.push("level".to_string());
                shape.push(num_levels);
            }
            dim_names.push("latitude".to_string());
            shape.push(nj);

            dim_names.push("longitude".to_string());
            shape.push(ni);

            let mut message_infos = Vec::new();
            for m in &msgs {
                let time_idx = unique_times
                    .iter()
                    .position(|t| t == &m.time_str)
                    .unwrap_or(0);
                let level_idx = unique_levels
                    .iter()
                    .position(|l| l == &m.level_val)
                    .unwrap_or(0);

                message_infos.push(GribMessageInfo {
                    byte_offset: m.byte_offset,
                    message_len: m.msg_len,
                    time_index: time_idx,
                    time_str: m.time_str.clone(),
                    level_index: level_idx,
                    level_value: m.level_val,
                    grid_shape: m.grid_shape,
                });
            }

            let first_full_name = msgs.first().and_then(|m| m.full_name.clone());
            let first_unit = msgs.first().and_then(|m| m.unit.clone());

            let mut attributes = HashMap::new();
            if let Some(ref long_name) = first_full_name {
                attributes.insert("long_name".to_string(), long_name.clone());
            }
            if let Some(ref unit_val) = first_unit {
                attributes.insert("units".to_string(), unit_val.clone());
            }

            let level_coords: Vec<String> = unique_levels
                .iter()
                .map(|lvl| lvl.map_or_else(|| "surface".to_string(), |v| format!("{v}")))
                .collect();

            let var = GribVariable {
                name: var_name.clone(),
                dim_names: dim_names.clone(),
                shape: shape.clone(),
                units: first_unit.clone(),
                long_name: first_full_name.clone(),
                attributes: attributes.clone(),
                time_coords: unique_times.clone(),
                level_coords,
                messages: message_infos,
            };

            let shape_u64: Vec<u64> = shape.iter().map(|&s| s as u64).collect();
            let estimated_uncompressed_bytes = calculate_variable_size_bytes(&shape_u64, "float32");

            var_info_list.push(VariableInfo {
                name: var_name.clone(),
                data_type: "float32".to_string(),
                shape: shape_u64.clone(),
                dimension_names: dim_names,
                chunk_shape: shape_u64,
                file_size: estimated_uncompressed_bytes,
                units: first_unit,
                long_name: first_full_name,
                time_coverage_start: unique_times.first().cloned(),
                time_coverage_end: unique_times.last().cloned(),
                temporal_resolution: None,
                attributes,
            });

            variables.insert(var_name, var);
        }

        let metadata = DatasetMetadata {
            name: dataset_name.to_string(),
            store_type: "GRIB / GRIB2 (Gribberish)".to_string(),
            variables: var_info_list,
            dimension_coordinates: dim_coords.clone(),
        };

        Ok((variables, dim_coords, metadata))
    }
}

impl BlockStore for GribberishBlockStore {
    fn backend_name(&self) -> &str {
        "GRIB / GRIB2 (Gribberish)"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        Ok(self.variables.keys().cloned().collect())
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        Ok(self.metadata.clone())
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        _on_progress: Option<&mut (dyn FnMut(u64) + Send)>,
    ) -> Result<OctantBlock, BlockStoreError> {
        self.fetch_block(request)
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        let var = self
            .variables
            .get(&request.variable)
            .ok_or_else(|| format!("Variable '{}' not found in GRIB store", request.variable))?;

        let file_bytes = std::fs::read(&self.file_path)
            .map_err(|e| format!("Failed to read GRIB file '{}': {e}", self.file_path))?;

        let rank = var.dim_names.len();
        let mut origin = Vec::with_capacity(rank);
        let mut block_shape = Vec::with_capacity(rank);

        for (i, sel) in request.selections.iter().enumerate() {
            let dim_len = var.shape.get(i).copied().unwrap_or(1);
            let (start, end) = sel.bounds();
            let start = start.min(dim_len.saturating_sub(1));
            let end = end.max(start.saturating_add(1)).min(dim_len);
            origin.push(start);
            block_shape.push(end.saturating_sub(start));
        }

        while origin.len() < rank {
            origin.push(0);
            let dim_len = var.shape.get(origin.len() - 1).copied().unwrap_or(1);
            block_shape.push(dim_len);
        }

        // Determine target time and level indices
        let (time_idx, level_idx) = match rank {
            2 => (0, 0),
            3 => {
                if var.dim_names[0] == "time" {
                    (origin[0], 0)
                } else {
                    (0, origin[0])
                }
            }
            4 => (origin[0], origin[1]),
            _ => (0, 0),
        };

        // Find the message corresponding to (time_idx, level_idx)
        let msg_info = match var
            .messages
            .iter()
            .find(|m| m.time_index == time_idx && m.level_index == level_idx)
        {
            Some(m) => m,
            None => {
                let fallback = var.messages.first().ok_or_else(|| {
                    format!(
                        "No GRIB message found for variable '{}' at time {time_idx}, level {level_idx}",
                        request.variable
                    )
                })?;
                log::warn!(
                    "Exact GRIB message match not found for variable '{}' at time_index={}, level_index={}; falling back to message at offset {}",
                    request.variable,
                    time_idx,
                    level_idx,
                    fallback.byte_offset
                );
                fallback
            }
        };

        let message = gribberish::message::read_message(&file_bytes, msg_info.byte_offset)
            .ok_or_else(|| {
                format!(
                    "Failed to parse GRIB message at offset {}",
                    msg_info.byte_offset
                )
            })?;

        let raw_values = message
            .data()
            .map_err(|e| format!("GRIB decompression error: {e}"))?;

        let (nj, ni) = msg_info.grid_shape;
        let total_grid_points = nj.saturating_mul(ni);
        if raw_values.len() < total_grid_points {
            log::warn!(
                "Decoded GRIB points ({}) is smaller than expected grid dimensions ({} x {} = {})",
                raw_values.len(),
                nj,
                ni,
                total_grid_points
            );
            return Err(format!(
                "Decoded point count {} is smaller than expected grid points {}",
                raw_values.len(),
                total_grid_points
            )
            .into());
        }

        // Spatial slice indices
        let (y_start, y_len, x_start, x_len) = match rank {
            2 => (origin[0], block_shape[0], origin[1], block_shape[1]),
            3 => (origin[1], block_shape[1], origin[2], block_shape[2]),
            4 => (origin[2], block_shape[2], origin[3], block_shape[3]),
            _ => (0, nj, 0, ni),
        };

        let y_end = (y_start + y_len).min(nj);
        let x_end = (x_start + x_len).min(ni);

        let out_nj = y_end.saturating_sub(y_start);
        let out_ni = x_end.saturating_sub(x_start);

        let out_len = out_nj.checked_mul(out_ni).unwrap_or(0);
        let mut f32_data = Vec::with_capacity(out_len);

        for r in y_start..y_end {
            let row_offset = r.saturating_mul(ni);
            for c in x_start..x_end {
                let idx = row_offset.saturating_add(c);
                if idx < raw_values.len() {
                    let val = raw_values[idx];
                    if val.is_nan() {
                        f32_data.push(f32::NAN);
                    } else {
                        f32_data.push(val as f32);
                    }
                } else {
                    f32_data.push(f32::NAN);
                }
            }
        }

        let mut coordinates: HashMap<String, Vec<f64>> = HashMap::new();
        for (name, coords) in &self.dimension_coordinates {
            if let (Some(first), Some(last)) = (coords.first(), coords.last())
                && let (Ok(f), Ok(l)) = (first.parse::<f64>(), last.parse::<f64>())
            {
                coordinates.insert(name.clone(), vec![f, l]);
            }
        }

        let attributes_json: serde_json::Map<String, serde_json::Value> = var
            .attributes
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();

        let mut dim_names = var.dim_names.clone();

        let raw_values = check_and_orient_block_grid(
            f32_data,
            &mut block_shape,
            &mut dim_names,
            &mut origin,
            &attributes_json,
            &coordinates,
        );

        let block = OctantBlock::new(
            request.variable.clone(),
            block_shape,
            dim_names,
            origin,
            raw_values,
            coordinates,
            var.attributes.clone(),
        );

        Ok(block)
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        let mut blocks = Vec::with_capacity(requests.len());
        for req in requests {
            blocks.push(self.fetch_block(req)?);
        }
        Ok(BlockResult::new(blocks))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_nonexistent_file() {
        let res = GribberishBlockStore::open_local("/path/that/definitely/does/not/exist.grib2");
        assert!(res.is_err());
        let err = res.err().unwrap().to_string();
        assert!(err.contains("File not found"));
    }

    #[test]
    fn test_open_directory_returns_error() {
        let temp_dir = std::env::temp_dir();
        let res = GribberishBlockStore::open_local(temp_dir.to_str().unwrap());
        assert!(res.is_err());
        let err = res.err().unwrap().to_string();
        assert!(err.contains("is a directory"));
    }

    #[test]
    fn test_index_invalid_bytes() {
        let dummy = b"NOT_A_VALID_GRIB_FILE_HEADER_DATA";
        let res = GribberishBlockStore::index_grib_file(dummy, "dummy_dataset");
        assert!(res.is_err());
        let err = res.err().unwrap().to_string();
        assert!(err.contains("No valid GRIB messages found"));
    }
}
