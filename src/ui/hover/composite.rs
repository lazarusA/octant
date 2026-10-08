//! Hover readout of a composite: its name, and the band behind each color channel with the
//! band's raw value at the hovered pixel.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use crate::app::OctantApp;
use crate::data::slicing::CompositeProbe;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::ui::hover::composite_bands::{BandNames, is_non_visible, visible_color};
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
    /// Cyan, magenta, yellow and black inks converted to RGB.
    Cmyk,
    /// Several channels, each tinted with its own color and added up.
    Overlay,
}

impl CompositeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::TrueColor => "True color",
            Self::FalseColor => "False color",
            Self::Rgb => "RGB composite",
            Self::Cmyk => "CMYK",
            Self::Overlay => "Channel overlay",
        }
    }
}

const RGB_LETTERS: [&str; 3] = ["R", "G", "B"];
const CMYK_LETTERS: [&str; 4] = ["C", "M", "Y", "K"];
/// Names an RGB band mapping from its three band names (`None` for unnamed bands): true
/// color when red, green and blue bands land on R, G and B; false color when a band outside
/// the visible range or a color in another slot is drawn; otherwise just an RGB composite,
/// e.g. for band codes like `B04` that name no color.
pub fn classify_rgb(names: [Option<&str>; 3]) -> CompositeKind {
    let [Some(r), Some(g), Some(b)] = names else {
        return CompositeKind::Rgb;
    };
    let colors = [r, g, b].map(visible_color);
    if colors == [Some(0), Some(1), Some(2)] {
        return CompositeKind::TrueColor;
    }
    let misplaced = (0..3).any(|slot| colors[slot].is_some_and(|c| c != slot));
    if misplaced || [r, g, b].into_iter().any(is_non_visible) {
        CompositeKind::FalseColor
    } else {
        CompositeKind::Rgb
    }
}

/// One composite channel: its row label (`R`, `C`, or an overlay channel's name), the global
/// band it reads, and the band's name for band mappings.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeChannel {
    pub label: Arc<str>,
    pub band: usize,
    pub band_name: Option<String>,
}

/// The composite the plot draws: its name and channels, resolved once per plot.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeLabels {
    pub kind: CompositeKind,
    pub channels: Vec<CompositeChannel>,
}

/// [`build_labels`], cached in egui temp memory until the dataset, variable or channel
/// mapping changes, so the hover and settings do no lookups or formatting per frame.
pub fn composite_labels(
    app: &OctantApp,
    ctx: &egui::Context,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
) -> Arc<CompositeLabels> {
    let mut hasher = DefaultHasher::new();
    app.plotted().store_target.hash(&mut hasher);
    let generations = (app.plotted().metadata_generation, app.coordinates_revision);
    (meta.is_some(), generations).hash(&mut hasher);
    var.map(|v| (&v.name, &v.shape, &v.dimension_names))
        .hash(&mut hasher);
    (is_overlay(app), cmyk_bands(app), rgb_bands(app)).hash(&mut hasher);
    for c in &app.layers.base.composite.channel_configs {
        (c.index, &c.name, c.visible).hash(&mut hasher);
    }
    let key = hasher.finish();
    let id = egui::Id::new("composite_labels");
    if let Some((cached_key, labels)) = ctx.data(|d| d.get_temp::<(u64, Arc<CompositeLabels>)>(id))
        && cached_key == key
    {
        return labels;
    }
    let labels = Arc::new(build_labels(app, meta, var));
    ctx.data_mut(|d| d.insert_temp(id, (key, Arc::clone(&labels))));
    labels
}

/// The composite's name and channels: visible overlay channels, the C, M, Y, K inks, or the
/// bands drawn as R, G and B.
pub fn build_labels(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
) -> CompositeLabels {
    if is_overlay(app) {
        let channels = app
            .layers
            .base
            .composite
            .channel_configs
            .iter()
            .filter(|c| c.visible);
        let channels = channels.map(|c| CompositeChannel {
            label: match c.name.trim() {
                "" => format!("Channel {}", c.index + 1).into(),
                name => name.into(),
            },
            band: c.index,
            band_name: None,
        });
        return CompositeLabels {
            kind: CompositeKind::Overlay,
            channels: channels.collect(),
        };
    }
    let names = BandNames::resolve(app, meta, var);
    let band = |(letter, band): (&str, usize)| CompositeChannel {
        label: letter.into(),
        band,
        band_name: Some(names.name(band)),
    };
    match cmyk_bands(app) {
        Some(inks) => CompositeLabels {
            kind: CompositeKind::Cmyk,
            channels: CMYK_LETTERS.into_iter().zip(inks).map(band).collect(),
        },
        None => {
            let bands = rgb_bands(app);
            CompositeLabels {
                kind: classify_rgb(bands.map(|b| names.real_name(b))),
                channels: RGB_LETTERS.into_iter().zip(bands).map(band).collect(),
            }
        }
    }
}

/// One row per composite channel at composite pixel `(px, py)`: `R  NIR 0.312` for band
/// mappings, `DAPI  1234` for overlays.
pub fn composite_fields(
    app: &OctantApp,
    labels: &CompositeLabels,
    units: &str,
    (px, py): (usize, usize),
) -> Vec<HoverField> {
    let probe = app.layers.base.data.composite_probe.as_ref();
    let raw = |band: usize| probe.and_then(|p| p.sample(band, px, py));
    labels
        .channels
        .iter()
        .map(|c| {
            let mut value = String::with_capacity(32);
            if let Some(name) = &c.band_name {
                value.push_str(name);
                value.push(' ');
            }
            write_raw(&mut value, raw(c.band), units);
            HoverField::new(Arc::clone(&c.label), value)
        })
        .collect()
}

/// Mirrors the projection's choice between the tinted overlay and the RGB band mapping.
fn is_overlay(app: &OctantApp) -> bool {
    !app.is_geotiff() && !app.layers.base.composite.channel_configs.is_empty()
}

/// The global C, M, Y and K bands when the plotted composite converts CMYK inks: from the
/// 2D composite's block, or for volumes, which keep no probe, from the dataset's tags.
fn cmyk_bands(app: &OctantApp) -> Option<[usize; 4]> {
    match app.layers.base.data.composite_probe.as_ref() {
        Some(probe) => probe.cmyk_channels(),
        None => (app.is_cmyk() && app.num_bands() >= 4).then_some([0, 1, 2, 3]),
    }
}

/// The global bands drawn as R, G and B, clamped into the loaded block like the slicer.
fn rgb_bands(app: &OctantApp) -> [usize; 3] {
    let resolve = |band: usize| {
        app.layers
            .base
            .data
            .composite_probe
            .as_ref()
            .map_or(band, |p: &CompositeProbe| p.resolve_channel(band))
    };
    app.layers.base.composite.rgb_channels.map(resolve)
}

/// Appends a channel's raw value (with `units`), or "No data", to `out`.
fn write_raw(out: &mut String, raw: Option<f32>, units: &str) {
    let Some(value) = raw else {
        out.push_str("No data");
        return;
    };
    let mut buf = [0u8; 32];
    out.push_str(crate::ui::hover::card::HoverValue::from_raw(value, None).format(&mut buf));
    if !units.is_empty() && !value.is_nan() {
        out.push(' ');
        out.push_str(units);
    }
}

/// The composite's name, after prepending one row per channel (its band and raw value at
/// `pixel`) to `entries` for flat 2D composites; `None` without a composite.
pub fn composite_rows(
    app: &OctantApp,
    ctx: &egui::Context,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    units: &str,
    pixel: Option<(usize, usize)>,
    entries: &mut Vec<HoverField>,
) -> Option<CompositeKind> {
    if !app.layers.base.composite.enabled {
        return None;
    }
    let labels = composite_labels(app, ctx, meta, var);
    if let Some(pixel) = pixel
        && app.layers.base.data.composite_probe.is_some()
    {
        entries.splice(0..0, composite_fields(app, &labels, units, pixel));
    }
    Some(labels.kind)
}
