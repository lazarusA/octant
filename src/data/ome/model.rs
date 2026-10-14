//! Octant's version-independent view of an OME-Zarr image and its channels.

use std::sync::Arc;

use super::axes::{AxisKind, OmeAxis};

/// One resolution level: its array path and the physical position of each
/// index along every axis (`translation + index * scale`).
#[derive(Debug, Clone, PartialEq)]
pub struct OmeLevel {
    /// The level array's path relative to its image group.
    pub path: String,
    pub scale: Vec<f64>,
    pub translation: Vec<f64>,
}

/// How a channel maps intensities onto its color (OMERO `family`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub enum ChannelFamily {
    #[default]
    Linear,
    Logarithmic,
    Exponential,
    Polynomial,
}

impl ChannelFamily {
    /// The family named by OMERO (`"linear"`, `"logarithmic"`, ...).
    pub fn parse(name: &str) -> Self {
        let n = name.trim();
        if n.eq_ignore_ascii_case("logarithmic") {
            Self::Logarithmic
        } else if n.eq_ignore_ascii_case("exponential") {
            Self::Exponential
        } else if n.eq_ignore_ascii_case("polynomial") {
            Self::Polynomial
        } else {
            Self::Linear
        }
    }
}

/// A channel's display window: the values it may span (`min`..`max`) and the
/// part shown from black to full color (`start`..`end`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelWindow {
    pub min: f32,
    pub max: f32,
    pub start: f32,
    pub end: f32,
}

/// One channel's OMERO rendering settings.
#[derive(Debug, Clone, PartialEq)]
pub struct OmeChannel {
    /// The channel's label, empty when it has none.
    pub label: String,
    pub color: [u8; 3],
    pub active: bool,
    pub window: Option<ChannelWindow>,
    pub inverted: bool,
    pub family: ChannelFamily,
    pub coefficient: f32,
}

/// A label image's colors and properties (`image-label`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OmeLabelInfo {
    /// `(label value, RGBA)`, sorted by value.
    pub colors: Vec<(u64, [u8; 4])>,
    /// `(label value, "key: value, ...")`, sorted by value: the properties as
    /// the hover card shows them, written once.
    pub properties_text: Vec<(u64, String)>,
}

impl OmeLabelInfo {
    /// The color recorded for `value`.
    pub fn color_of(&self, value: u64) -> Option<[u8; 4]> {
        self.colors
            .binary_search_by_key(&value, |(v, _)| *v)
            .ok()
            .map(|i| self.colors[i].1)
    }

    /// The properties of `value` as one line (`"class: nucleus, area: 31"`).
    pub fn properties_line(&self, value: u64) -> Option<&str> {
        self.properties_text
            .binary_search_by_key(&value, |(v, _)| *v)
            .ok()
            .map(|i| self.properties_text[i].1.as_str())
    }
}

/// An OME-Zarr multiscale image.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OmeImage {
    /// The image group's path in the store (`""` for the root).
    pub group: String,
    pub name: Option<String>,
    pub axes: Vec<OmeAxis>,
    /// Resolution levels, finest first.
    pub levels: Vec<OmeLevel>,
    pub channels: Vec<OmeChannel>,
    pub default_t: Option<usize>,
    pub default_z: Option<usize>,
    /// OMERO `model: "greyscale"`: show one channel with a gray colormap.
    pub greyscale: bool,
    /// Names of the label images in this image's `labels/` group.
    pub labels: Vec<String>,
    /// Set when this image is itself a label image.
    pub label_info: Option<Arc<OmeLabelInfo>>,
}

impl OmeImage {
    /// Index of the first axis of `kind`.
    pub fn axis_of_kind(&self, kind: AxisKind) -> Option<usize> {
        self.axes.iter().position(|a| a.kind == kind)
    }

    /// The channel a greyscale image (`model: "greyscale"`) shows first: the
    /// first active channel, else the first.
    pub fn greyscale_channel(&self) -> Option<usize> {
        if !self.greyscale || self.axis_of_kind(AxisKind::Channel).is_none() {
            return None;
        }
        Some(self.channels.iter().position(|c| c.active).unwrap_or(0))
    }

    /// Index of the z axis: the third-last space axis, when there are three.
    pub fn z_axis(&self) -> Option<usize> {
        (0..self.axes.len())
            .rev()
            .filter(|&i| self.axes[i].kind == AxisKind::Space)
            .nth(2)
    }
}

/// How a greyscale image (`model: "greyscale"`) shows the channel at
/// `channel`: its window and whether it is inverted.
pub fn greyscale_display(
    image: &OmeImage,
    channel: usize,
) -> Option<(Option<ChannelWindow>, bool)> {
    image.greyscale_channel()?;
    let ch = image.channels.get(channel)?;
    Some((ch.window, ch.inverted))
}

/// A variable's place in an OME-Zarr image: the image and which level it is.
#[derive(Debug, Clone, PartialEq)]
pub struct OmeRef {
    pub image: Arc<OmeImage>,
    pub level: usize,
}

impl OmeRef {
    pub fn level(&self) -> Option<&OmeLevel> {
        self.image.levels.get(self.level)
    }

    /// The physical step between indices of axis `axis` at this level.
    pub fn scale(&self, axis: usize) -> Option<f64> {
        self.level()?.scale.get(axis).copied()
    }
}
