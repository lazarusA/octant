//! CF (Climate and Forecast) metadata convention parsing and attribute resolution.

use std::collections::HashMap;

/// Common CF convention metadata and dimensions parsed from attributes map / metadata.
#[derive(Debug, Clone, Default)]
pub struct ParsedCfAttributes {
    pub attributes: HashMap<String, String>,
    pub units: Option<String>,
    pub long_name: Option<String>,
    pub time_coverage_start: Option<String>,
    pub time_coverage_end: Option<String>,
    pub temporal_resolution: Option<String>,
    pub array_dimensions: Option<Vec<String>>,
}

impl ParsedCfAttributes {
    /// Parses CF metadata and dimension names from any key-value JSON map or iterator.
    pub fn from_json_map<'a, I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a String, &'a serde_json::Value)>,
    {
        let mut attributes = HashMap::new();
        let mut units = None;
        let mut long_name = None;
        let mut time_coverage_start = None;
        let mut time_coverage_end = None;
        let mut temporal_resolution = None;
        let mut array_dimensions = None;

        for (k, v_json) in iter {
            let val_str = if let Some(s) = v_json.as_str() {
                s.to_string()
            } else {
                v_json.to_string()
            };
            attributes.insert(k.clone(), val_str.clone());

            match k.as_str() {
                "units" => units = Some(val_str),
                "long_name" => long_name = Some(val_str),
                "time_coverage_start" => time_coverage_start = Some(val_str),
                "time_coverage_end" => time_coverage_end = Some(val_str),
                "temporal_resolution" | "time_period" => temporal_resolution = Some(val_str),
                "_ARRAY_DIMENSIONS" => {
                    if let Some(arr) = v_json.as_array() {
                        array_dimensions = Some(
                            arr.iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    s.as_str()
                                        .map(|str_v| str_v.to_string())
                                        .unwrap_or_else(|| format!("dim_{i}"))
                                })
                                .collect(),
                        );
                    }
                }
                _ => {}
            }
        }

        Self {
            attributes,
            units,
            long_name,
            time_coverage_start,
            time_coverage_end,
            temporal_resolution,
            array_dimensions,
        }
    }

    /// Resolves final dimension names using explicit names, `_ARRAY_DIMENSIONS`, or rank defaults.
    pub fn resolve_dimension_names(
        &self,
        explicit_dimension_names: Option<&[Option<String>]>,
        rank: usize,
    ) -> Vec<String> {
        if let Some(names) = explicit_dimension_names {
            return names
                .iter()
                .enumerate()
                .map(|(i, n)| n.clone().unwrap_or_else(|| format!("dim_{i}")))
                .collect();
        }

        if let Some(ref dims) = self.array_dimensions {
            return dims.clone();
        }

        default_dimension_names_for_rank(rank)
    }
}

/// Helper to find the first matching attribute value from a slice of candidate key aliases.
#[inline]
pub fn find_first_attr<'a>(
    attributes: &'a HashMap<String, String>,
    keys: &[&str],
) -> Option<&'a str> {
    for &k in keys {
        if let Some(v) = attributes.get(k) {
            return Some(v.as_str());
        }
    }
    None
}

/// Merges parent group attributes into target attributes map without overriding existing keys.
#[inline]
pub fn merge_parent_attributes(
    target: &mut HashMap<String, String>,
    parent: &HashMap<String, String>,
) {
    for (k, v) in parent {
        target.entry(k.clone()).or_insert_with(|| v.clone());
    }
}

/// Resolves inherited attributes by walking up all ancestor group paths.
pub fn resolve_ancestor_attributes(
    group_attrs: &HashMap<String, HashMap<String, String>>,
    var_path: &str,
) -> HashMap<String, String> {
    let mut merged = HashMap::new();
    for ancestor in crate::utils::path::ancestor_paths(var_path) {
        if let Some(attrs) = group_attrs.get(ancestor) {
            merge_parent_attributes(&mut merged, attrs);
        }
    }
    merged
}

/// Returns standard fallback dimension names for a given tensor rank.
pub fn default_dimension_names_for_rank(rank: usize) -> Vec<String> {
    match rank {
        1 => vec!["x".to_string()],
        2 => vec!["lat".to_string(), "lon".to_string()],
        3 => vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
        4 => vec![
            "time".to_string(),
            "level".to_string(),
            "lat".to_string(),
            "lon".to_string(),
        ],
        _ => (0..rank).map(|i| format!("dim_{}", i)).collect(),
    }
}
