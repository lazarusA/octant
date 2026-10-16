//! Walks an OME-Zarr store from its root: images and their label images,
//! wells and their fields, plates, and `bioformats2raw` series.

use std::cell::Cell;
use std::sync::Arc;

use serde_json::{Map, Value};

use super::image::image_from_group;
use super::model::OmeImage;
use super::parse::{ParsedGroup, is_ome_attributes, parse_group};
use super::plate::{OmeDataset, plate_info, well_info};
use super::reader::{MetaReader, Pending, join};
use super::variables::image_variables;
use crate::data::metadata::VariableInfo;

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
        Ok(Some(attrs)) if is_ome_attributes(&attrs) => attrs,
        Ok(_) => return Outcome::NotOme,
    };
    let mut walk = Walk {
        reader,
        found: Discovery::default(),
        pending: false,
    };
    walk.visit_attrs(root, attrs, None, 0);
    let mut found = walk.found;
    // Sorted for `OmeDataset::group_name`'s binary search.
    found.dataset.group_names.sort_by(|a, b| a.0.cmp(&b.0));
    found.dataset.group_names.dedup_by(|a, b| a.0 == b.0);
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

pub(super) struct Walk<'r, 'a> {
    pub(super) reader: &'r MetaReader<'a>,
    pub(super) found: Discovery,
    pub(super) pending: bool,
}

impl Walk<'_, '_> {
    pub(super) fn visit(&mut self, path: &str, name: Option<&str>, depth: usize) {
        match self.reader.group_attributes(path) {
            Err(Pending) => self.pending = true,
            Ok(Some(attrs)) => self.visit_attrs(path, attrs, name, depth),
            Ok(None) => {}
        }
    }

    fn parse(&mut self, path: &str, attrs: Arc<Map<String, Value>>) -> Option<ParsedGroup> {
        if !is_ome_attributes(&attrs) {
            return None;
        }
        // Level shapes are read only for metadata that needs them (legacy
        // versions, missing transforms).
        let waiting = Cell::new(false);
        let reader = self.reader;
        let shapes = |level: &str| match reader.array_shape(&join(path, level)) {
            Ok(shape) => shape,
            Err(Pending) => {
                waiting.set(true);
                None
            }
        };
        let parsed = parse_group(Arc::unwrap_or_clone(attrs), &shapes);
        if waiting.get() {
            self.pending = true;
            return None;
        }
        let parsed = parsed?;
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
        attrs: Arc<Map<String, Value>>,
        name: Option<&str>,
        depth: usize,
    ) {
        if depth > MAX_DEPTH {
            return;
        }
        let Some(mut parsed) = self.parse(path, attrs) else {
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
            self.add_image(image, depth);
        }
        if let Some(plate) = parsed.fields.plate.take()
            && self.found.dataset.plate.is_none()
        {
            self.found.dataset.plate = Some(Arc::new(plate_info(plate)));
        }
        if let Some(well) = parsed.fields.well.take() {
            let well = well_info(well);
            for field in &well.fields {
                self.visit(&join(path, &field.path), None, depth + 1);
            }
            if self.found.dataset.well.is_none() {
                self.found.dataset.well = Some(Arc::new(well));
            }
        }
    }

    fn add_image(&mut self, image: OmeImage, depth: usize) {
        let image = Arc::new(image);
        match image_variables(self.reader, &image) {
            Ok(mut vars) => {
                crate::data::multiscale::ome::link_image(&image, &mut vars);
                self.found.variables.extend(vars);
            }
            Err(Pending) => self.pending = true,
        }
        let labels = join(&image.group, "labels");
        for label in &image.labels {
            self.visit(&join(&labels, label), None, depth + 1);
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
        self.parse(&path, attrs)
            .and_then(|p| p.fields.labels)
            .unwrap_or_default()
            .into_iter()
            .map(|l| l.trim_matches('/').to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }
}
