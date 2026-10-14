//! `bioformats2raw` containers: their series, listed or probed, and the
//! series names of `OME/METADATA.ome.xml`.

use std::collections::HashMap;

use super::discover::Walk;
use super::ome_xml::series_names;
use super::reader::{Lookup, Pending, join};

/// Series probed in a container that lists none.
const MAX_SERIES: usize = 4096;
/// Series asked for together while probing on the web.
const SERIES_BATCH: usize = 64;

impl Walk<'_, '_> {
    /// The container's series: its `series` list (on its `OME` group or
    /// itself), else groups `0`, `1`, ... up to the first one missing.
    /// Names come from `OME/METADATA.ome.xml`.
    pub(super) fn visit_series(&mut self, path: &str, listed: Option<&[String]>, depth: usize) {
        // The spec lists them on the `OME` group; some writers on the root.
        let listed = match listed {
            Some(listed) => Some(listed.to_vec()),
            None => match self.reader.group_attributes(&join(path, "OME")) {
                Err(Pending) => {
                    self.pending = true;
                    return;
                }
                Ok(attrs) => attrs.as_deref().and_then(series_of),
            },
        };
        let series = match listed {
            Some(listed) => listed,
            None => match self.probe_series(path) {
                Some(series) => series,
                // Series are visited once probing is done, not on every pass.
                None => return,
            },
        };
        // Names only matter once there is a series (an HCS plate has none).
        let mut names = if series.is_empty() {
            HashMap::new()
        } else {
            self.series_names(path)
        };
        for (i, group) in series.iter().enumerate() {
            let group = join(path, group);
            let name = names.remove(&i);
            self.name_group(&group, name.as_deref());
            self.visit(&group, name.as_deref(), depth + 1);
        }
    }

    /// Groups `0`, `1`, ... up to the first one missing; `None` while some
    /// are still being fetched (web), the next [`SERIES_BATCH`] together.
    fn probe_series(&mut self, path: &str) -> Option<Vec<String>> {
        let mut series = Vec::new();
        for i in 0..MAX_SERIES {
            match self.reader.has_group(&join(path, &i.to_string())) {
                Ok(true) => series.push(i.to_string()),
                Ok(false) => break,
                Err(Pending) => {
                    self.pending = true;
                    for next in i + 1..i + SERIES_BATCH {
                        let _ = self.reader.has_group(&join(path, &next.to_string()));
                    }
                    return None;
                }
            }
        }
        Some(series)
    }

    /// Series names from `OME/METADATA.ome.xml`, by series index.
    fn series_names(&mut self, path: &str) -> HashMap<usize, String> {
        match self.reader.get(&join(path, "OME/METADATA.ome.xml")) {
            Lookup::Found(xml) => series_names(&xml),
            Lookup::Pending => {
                self.pending = true;
                HashMap::new()
            }
            Lookup::Absent => HashMap::new(),
        }
    }

    fn name_group(&mut self, path: &str, name: Option<&str>) {
        if let Some(name) = name {
            self.found
                .dataset
                .group_names
                .push((path.to_string(), name.to_string()));
        }
    }
}

/// The `series` list of an `OME` group's attributes (free, or in 0.5's `ome`).
fn series_of(attrs: &serde_json::Map<String, serde_json::Value>) -> Option<Vec<String>> {
    let ome = attrs
        .get("ome")
        .and_then(serde_json::Value::as_object)
        .unwrap_or(attrs);
    ome.get("series")?
        .as_array()?
        .iter()
        .map(|s| s.as_str().map(str::to_string))
        .collect()
}
