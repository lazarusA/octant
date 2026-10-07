//! Hover readout of a composite: its name, and the band behind each color channel with the
//! band's raw value at the hovered pixel.

use crate::app::OctantApp;
use crate::data::slicing::CompositeProbe;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::ui::hover::field::HoverField;

/// A band combination, named the way GIS tools name them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositeKind {
    /// The red, green and blue bands drawn as red, green and blue.
    TrueColor,
    /// Any other mapping of named bands, e.g. NIR, Red, Green.
    FalseColor,
    /// Bands without names to tell the two apart.
    Rgb,
    /// Several channels, each tinted with its own color and added up.
    Overlay,
}

impl CompositeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::TrueColor => "True color",
            Self::FalseColor => "False color",
            Self::Rgb => "RGB composite",
            Self::Overlay => "Channel overlay",
        }
    }
}

const RGB_LETTERS: [&str; 3] = ["R", "G", "B"];
const TRUE_COLOR_BANDS: [&str; 3] = ["red", "green", "blue"];

/// Names an RGB band mapping from its three band names (`None` for unnamed bands).
pub fn classify_rgb(names: [Option<&str>; 3]) -> CompositeKind {
    let [Some(r), Some(g), Some(b)] = names else {
        return CompositeKind::Rgb;
    };
    let is_true = [r, g, b]
        .iter()
        .zip(TRUE_COLOR_BANDS)
        .all(|(name, band)| name.trim().eq_ignore_ascii_case(band));
    if is_true {
        CompositeKind::TrueColor
    } else {
        CompositeKind::FalseColor
    }
}

/// The composite the plot draws, as the hover card and settings name it.
pub fn composite_kind(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
) -> CompositeKind {
    if is_overlay(app) {
        return CompositeKind::Overlay;
    }
    let bands = BandNames::resolve(app, meta, var);
    classify_rgb(rgb_bands(app).map(|band| bands.real_name(band)))
}

/// One row per composite channel at composite pixel `(px, py)`: `R  NIR 0.312` for band
/// mappings, `DAPI  1234` for overlays.
pub fn composite_fields(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    units: &str,
    (px, py): (usize, usize),
) -> Vec<HoverField> {
    let probe = app.composite_probe.as_ref();
    let raw = |band: usize| probe.and_then(|p| p.sample(band, px, py));
    if is_overlay(app) {
        return app
            .composite_channel_configs
            .iter()
            .filter(|c| c.visible)
            .map(|c| {
                let name = match c.name.trim() {
                    "" => format!("Channel {}", c.index + 1),
                    name => name.to_string(),
                };
                HoverField::new(name, format_raw(raw(c.index), units))
            })
            .collect();
    }
    let bands = BandNames::resolve(app, meta, var);
    RGB_LETTERS
        .iter()
        .zip(rgb_bands(app))
        .map(|(letter, band)| {
            let value = format_raw(raw(band), units);
            HoverField::new(*letter, format!("{} {value}", bands.name(band)))
        })
        .collect()
}

/// Mirrors the projection's choice between the tinted overlay and the RGB band mapping.
fn is_overlay(app: &OctantApp) -> bool {
    !app.is_geotiff() && !app.composite_channel_configs.is_empty()
}

/// The global bands drawn as R, G and B, clamped into the loaded block like the slicer.
fn rgb_bands(app: &OctantApp) -> [usize; 3] {
    let resolve = |band: usize| {
        app.composite_probe
            .as_ref()
            .map_or(band, |p: &CompositeProbe| p.resolve_channel(band))
    };
    app.rgb_composite_channels.map(resolve)
}

fn format_raw(raw: Option<f32>, units: &str) -> String {
    let Some(value) = raw else {
        return "No data".to_string();
    };
    let mut buf = [0u8; 32];
    let text = crate::ui::hover::card::HoverValue::from_raw(value, None).format(&mut buf);
    if units.is_empty() || value.is_nan() {
        text.to_string()
    } else {
        format!("{text} {units}")
    }
}

/// Labels of the channel dimension, when the dataset stores one per band.
struct BandNames<'a> {
    dim: &'a str,
    labels: Option<&'a [String]>,
}

impl<'a> BandNames<'a> {
    fn resolve(
        app: &OctantApp,
        meta: Option<&'a DatasetMetadata>,
        var: Option<&'a VariableInfo>,
    ) -> Self {
        let c_dim = app.channel_dim_index().unwrap_or(0);
        let dim = var
            .and_then(|v| v.dimension_names.get(c_dim))
            .map_or("band", String::as_str);
        let count = var.and_then(|v| v.shape.get(c_dim)).copied();
        let labels = meta
            .and_then(|m| m.get_dim_coords(var.map(|v| v.name.as_str()), dim))
            .filter(|l| count.is_some_and(|n| l.len() as u64 == n));
        Self { dim, labels }
    }

    /// The stored name of `band`, ignoring generated `Band N` placeholders.
    fn real_name(&self, band: usize) -> Option<&'a str> {
        let name = self.labels?.get(band)?.trim();
        let placeholder = name
            .strip_prefix("Band ")
            .is_some_and(|n| n.parse::<usize>().is_ok());
        (!name.is_empty() && !placeholder).then_some(name)
    }

    /// The name shown for `band`: its label, else the dimension name and 1-based index.
    fn name(&self, band: usize) -> String {
        match self.labels.and_then(|l| l.get(band)).map(|n| n.trim()) {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => {
                let mut chars = self.dim.chars();
                let first = chars.next().map(|c| c.to_ascii_uppercase());
                format!("{}{} {}", first.unwrap_or('B'), chars.as_str(), band + 1)
            }
        }
    }
}
