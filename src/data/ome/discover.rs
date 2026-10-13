//! Walks an OME-Zarr store from its root: images and their label images,
//! wells and their fields, plates, and `bioformats2raw` series.

use std::collections::HashMap;
use std::sync::Arc;

use super::image::image_from_group;
use super::model::OmeImage;
use super::ome_xml::series_names;
use super::parse::{ParsedGroup, level_paths, parse_group};
use super::plate::{OmeDataset, plate_info, well_info};
use super::reader::{Lookup, MetaReader, Pending, join};
use super::variables::image_variables;
use crate::data::metadata::VariableInfo;

/// Series probed in a container that lists none.
const MAX_SERIES: usize = 4096;
/// Series asked for together while probing on the web.
const SERIES_BATCH: usize = 64;
/// Nesting guard (container -> well -> image -> labels -> label image).
const MAX_DEPTH: usize = 6;

/// What an OME-Zarr store holds.
#[derive(Debug, Default)]
pub struct Discovery {
    pub variables: Vec<VariableInfo>,
    pub dataset: OmeDataset,
    /// What did not conform, per group (English, for the log).
    pub warnings: Vec<String>,
}

/// The result of one walk.
#[derive(Debug)]
pub enum Outcome {
    /// The root is not an OME-Zarr group.
    NotOme,
    /// Keys are still being fetched (web): walk again once they arrive.
    Pending,
    Found(Discovery),
}

/// Walks the store under `root` (`""` for the store's root).
pub fn discover(reader: &MetaReader<'_>, root: &str) -> Outcome {
    let attrs = match reader.group_attributes(root) {
        Err(Pending) => return Outcome::Pending,
        Ok(Some(attrs)) if super::parse::is_ome_attributes(&attrs) => attrs,
        Ok(_) => return Outcome::NotOme,
    };
    let mut walk = Walk {
        reader,
        found: Discovery::default(),
        pending: false,
    };
    walk.visit_attrs(root, attrs, None, 0);
    let found = walk.found;
    let empty =
        found.variables.is_empty() && found.dataset.plate.is_none() && found.dataset.well.is_none();
    if walk.pending {
        Outcome::Pending
    } else if empty {
        // Nothing OME-Zarr to show: the store opens as plain Zarr.
        Outcome::NotOme
    } else {
        Outcome::Found(found)
    }
}

struct Walk<'r, 'a> {
    reader: &'r MetaReader<'a>,
    found: Discovery,
    pending: bool,
}

impl Walk<'_, '_> {
    fn visit(&mut self, path: &str, name: Option<&str>, depth: usize) {
        match self.reader.group_attributes(path) {
            Err(Pending) => self.pending = true,
            Ok(Some(attrs)) => self.visit_attrs(path, attrs, name, depth),
            Ok(None) => {}
        }
    }

    fn parse(
        &mut self,
        path: &str,
        attrs: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<ParsedGroup> {
        let mut shapes = HashMap::new();
        for level in level_paths(attrs) {
            match self.reader.array_shape(&join(path, &level)) {
                Err(Pending) => self.pending = true,
                Ok(Some(shape)) => {
                    shapes.insert(level, shape);
                }
                Ok(None) => {}
            }
        }
        if self.pending {
            return None;
        }
        let parsed = parse_group(attrs, &|p| shapes.get(p.trim_matches('/')).cloned())?;
        let at = if path.is_empty() { "/" } else { path };
        self.found
            .warnings
            .extend(parsed.warnings.iter().map(|w| format!("{at}: {w}")));
        if self.found.dataset.version.is_empty() {
            self.found.dataset.version = parsed.version.clone();
        }
        Some(parsed)
    }

    fn visit_attrs(
        &mut self,
        path: &str,
        attrs: serde_json::Map<String, serde_json::Value>,
        name: Option<&str>,
        depth: usize,
    ) {
        if depth > MAX_DEPTH {
            return;
        }
        let Some(parsed) = self.parse(path, &attrs) else {
            return;
        };
        if let Some(layout) = &parsed.fields.bioformats2raw {
            self.visit_series(path, layout.series.as_deref(), depth);
        }
        if let Some(mut image) = image_from_group(path, &parsed) {
            if image.name.is_none() {
                image.name = name.map(str::to_string);
            }
            if image.label_info.is_none() {
                image.labels = self.label_names(path);
            }
            self.add_image(image, &parsed.version, depth);
        }
        if let Some(plate) = &parsed.fields.plate
            && self.found.dataset.plate.is_none()
        {
            self.found.dataset.plate = Some(Arc::new(plate_info(plate)));
        }
        if let Some(well) = &parsed.fields.well {
            let well = well_info(well);
            for field in &well.fields {
                self.visit(&join(path, &field.path), None, depth + 1);
            }
            if self.found.dataset.well.is_none() {
                self.found.dataset.well = Some(Arc::new(well));
            }
        }
    }

    fn add_image(&mut self, image: OmeImage, version: &str, depth: usize) {
        let labels: Vec<String> = image.labels.clone();
        let group = image.group.clone();
        let image = Arc::new(image);
        match image_variables(self.reader, &image, version) {
            Ok(vars) => self.found.variables.extend(vars),
            Err(Pending) => self.pending = true,
        }
        for label in labels {
            self.visit(&join(&join(&group, "labels"), &label), None, depth + 1);
        }
    }

    /// Names in the image's `labels/` group.
    fn label_names(&mut self, image: &str) -> Vec<String> {
        let path = join(image, "labels");
        let attrs = match self.reader.group_attributes(&path) {
            Err(Pending) => {
                self.pending = true;
                return Vec::new();
            }
            Ok(Some(attrs)) => attrs,
            Ok(None) => return Vec::new(),
        };
        self.parse(&path, &attrs)
            .and_then(|p| p.fields.labels)
            .unwrap_or_default()
            .into_iter()
            .map(|l| l.trim_matches('/').to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }

    /// The container's series: its `series` list, else groups `0`, `1`, ...
    /// until one is missing. Names come from `OME/METADATA.ome.xml`.
    fn visit_series(&mut self, path: &str, listed: Option<&[String]>, depth: usize) {
        let series = match listed {
            Some(listed) => listed.to_vec(),
            None => self.probe_series(path),
        };
        // Names only matter once there is a series (an HCS plate has none).
        let names = if series.is_empty() {
            Vec::new()
        } else {
            self.series_names(path)
        };
        for (i, group) in series.iter().enumerate() {
            let group = join(path, group);
            let name = names.get(i).cloned().flatten();
            self.name_group(&group, name.as_deref());
            self.visit(&group, name.as_deref(), depth + 1);
        }
    }

    /// Groups `0`, `1`, ... up to the first one missing. On the web, the next
    /// [`SERIES_BATCH`] groups are asked for together.
    fn probe_series(&mut self, path: &str) -> Vec<String> {
        let mut series = Vec::new();
        for i in 0..MAX_SERIES {
            match self.reader.group_attributes(&join(path, &i.to_string())) {
                Ok(Some(_)) => series.push(i.to_string()),
                Ok(None) => break,
                Err(Pending) => {
                    self.pending = true;
                    for next in i + 1..i + SERIES_BATCH {
                        let _ = self.reader.group_attributes(&join(path, &next.to_string()));
                    }
                    break;
                }
            }
        }
        series
    }

    /// Series names from `OME/METADATA.ome.xml`, by series index.
    fn series_names(&mut self, path: &str) -> Vec<Option<String>> {
        match self.reader.get(&join(path, "OME/METADATA.ome.xml")) {
            Lookup::Found(xml) => series_names(&xml),
            Lookup::Pending => {
                self.pending = true;
                Vec::new()
            }
            Lookup::Absent => Vec::new(),
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
