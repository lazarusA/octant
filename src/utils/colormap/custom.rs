//! User-defined colormaps built on the fly with `colorgrad` and baked into the
//! same LUT representation as the bundled catalog.

use super::catalog::ColormapEntry;
use super::kind::ColormapKind;
use super::lut::Lut;
use colorgrad::{BlendMode, Gradient, GradientBuilder};
use serde::{Deserialize, Serialize};

/// Family prefix of every custom colormap key.
pub const CUSTOM_FAMILY_KEY: &str = "custom";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Interpolation {
    #[default]
    Linear,
    /// Smooth B-spline through the stops (does not pass through inner colors).
    Basis,
    /// Smooth Catmull-Rom spline passing through every stop.
    CatmullRom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BlendSpace {
    Rgb,
    LinearRgb,
    /// Perceptual blending; the default since it avoids muddy midpoints.
    #[default]
    Oklab,
    Lab,
}

impl Interpolation {
    pub const ALL: [Self; 3] = [Self::Linear, Self::Basis, Self::CatmullRom];
    pub fn label(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Basis => "B-spline",
            Self::CatmullRom => "Catmull-Rom",
        }
    }
}

impl BlendSpace {
    pub const ALL: [Self; 4] = [Self::Oklab, Self::Lab, Self::LinearRgb, Self::Rgb];
    pub fn label(self) -> &'static str {
        match self {
            Self::Rgb => "sRGB",
            Self::LinearRgb => "Linear RGB",
            Self::Oklab => "Oklab",
            Self::Lab => "CIE Lab",
        }
    }
    fn mode(self) -> BlendMode {
        match self {
            Self::Rgb => BlendMode::Rgb,
            Self::LinearRgb => BlendMode::LinearRgb,
            Self::Oklab => BlendMode::Oklab,
            Self::Lab => BlendMode::Lab,
        }
    }
}

/// Persisted description of a user colormap. Missing fields take their defaults,
/// so specs saved by older versions still load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomColormapSpec {
    pub name: String,
    /// CSS gradient syntax: comma separated colors (hex, `rgb()`, `hsl()`, names),
    /// optionally with positions, e.g. `"navy, 30%, gold, white"` or `"#000 0%, red 80%, #fff"`.
    pub colors: String,
    pub interpolation: Interpolation,
    pub blend: BlendSpace,
    /// Number of hard color classes; `0` keeps the gradient continuous.
    pub classes: u16,
}

impl Default for CustomColormapSpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            colors: "#0d0887, #cc4778, #f0f921".into(),
            interpolation: Interpolation::default(),
            blend: BlendSpace::default(),
            classes: 0,
        }
    }
}

impl CustomColormapSpec {
    pub fn key(&self) -> String {
        format!("{CUSTOM_FAMILY_KEY}:{}", self.name.trim())
    }

    /// Whether `key` is this spec's registry key, without building it.
    pub fn has_key(&self, key: &str) -> bool {
        key.strip_prefix(CUSTOM_FAMILY_KEY)
            .and_then(|rest| rest.strip_prefix(':'))
            == Some(self.name.trim())
    }

    /// Bakes the spec into a 256-entry LUT.
    pub fn build_lut(&self) -> Result<Box<Lut>, String> {
        let mut builder = GradientBuilder::new();
        builder.mode(self.blend.mode()).css(&self.colors);
        let gradient: Box<dyn Gradient> = match self.interpolation {
            Interpolation::Linear => builder
                .build::<colorgrad::LinearGradient>()
                .map(|g| g.boxed()),
            Interpolation::Basis => builder
                .build::<colorgrad::BasisGradient>()
                .map(|g| g.boxed()),
            Interpolation::CatmullRom => builder
                .build::<colorgrad::CatmullRomGradient>()
                .map(|g| g.boxed()),
        }
        .map_err(|e| e.to_string())?;
        let gradient = if self.classes > 1 {
            gradient.sharp(self.classes, 0.0).boxed()
        } else {
            gradient
        };
        Ok(super::lut::bake(gradient.as_ref()))
    }

    /// Validates the spec and builds a registry entry.
    pub fn to_entry(&self) -> Result<ColormapEntry, String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("Name is required".into());
        }
        let kind = if self.classes > 1 {
            ColormapKind::Categorical
        } else {
            ColormapKind::Sequential
        };
        Ok(ColormapEntry {
            key: self.key(),
            name: name.to_string(),
            family: None,
            kind,
            lut: self.build_lut()?,
            smooth: if self.classes > 1 {
                Some(
                    Self {
                        classes: 0,
                        ..self.clone()
                    }
                    .build_lut()?,
                )
            } else {
                None
            },
        })
    }
}
