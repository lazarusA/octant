//! Which settings change the plot on the canvas: one table, read by every
//! settings section, so the panel shows only what the renderer honors.

use crate::app::OctantApp;
use crate::plots::PlotType;

/// Volume algorithms (`VolumeUniformParams::algorithm`) the table tells apart.
const VOLUME_DVR: u32 = 0;
const VOLUME_LABEL_SURFACE: u32 = 4;
const VOLUME_ADDITIVE_RGBA: u32 = 6;
const VOLUME_INDEXED_RGBA: u32 = 7;

const COMPOSITE: &str = "Colors come from the composite channels.";
const CUSTOM_COLOR: &str = "Custom Color draws every point in one color.";
const ALL_SERIES: &str = "All Lines Series colors each line by its index.";
const INDEXED: &str = "Indexed RGBA colors voxels by their label index.";
const INDEXED_MISSING: &str = "Indexed RGBA skips missing voxels.";
const CLASSIC_OPAQUE: &str = "Only the DVR algorithm draws translucent colors.";
const NOT_TRANSLUCENT: &str = "Takes effect with Opacity below 1 or an alpha curve.";
const COMPOSITE_OPAQUE: &str = "Composites are drawn opaque.";
const LABEL_OPAQUE: &str = "Label surfaces are always drawn opaque.";
const ALWAYS_BLENDS: &str = "This algorithm always blends along the ray.";

/// Whether a setting changes the current plot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Support {
    /// The setting takes effect.
    Yes,
    /// Another setting the user chose takes its place, for this reason.
    Overridden(&'static str),
    /// The plot type has no use for it.
    No,
}

impl Support {
    pub(crate) fn is_yes(self) -> bool {
        self == Support::Yes
    }

    /// Why the setting has no effect, when another setting overrides it.
    pub(crate) fn reason(self) -> Option<&'static str> {
        match self {
            Support::Overridden(reason) => Some(reason),
            Support::Yes | Support::No => None,
        }
    }
}

/// What decides which settings apply.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PlotState {
    pub plot_type: PlotType,
    pub volume_algorithm: u32,
    pub composite: bool,
    pub line_custom_color: bool,
    pub line_all_series: bool,
    /// Opacity below 1 or an alpha curve.
    pub translucent: bool,
    /// Coastlines fit the data (not microscopy, not a channel overlay).
    pub geographic: bool,
}

/// Support of every setting whose effect depends on the plot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OptionSupport {
    /// Color range min and max (also the line plot's value axis).
    pub color_range: Support,
    /// Scale, categorical colors, and the low and high clip colors.
    pub color_mapping: Support,
    pub nan_color: Support,
    /// Opacity and the alpha curve.
    pub opacity: Support,
    /// Order-independent transparency of meshes and point clouds.
    pub transparency: Support,
    /// The volume's own transparency toggle.
    pub volume_transparency: Support,
    pub coastlines: Support,
    /// 2D Aggregation (pyramid resampling).
    pub aggregation: Support,
    /// Auto rotate and reset view.
    pub camera: Support,
}

impl PlotState {
    /// The state of layer `id`'s plot: the canvas plot for the base layer, a
    /// plain heatmap for an overlay.
    pub(crate) fn of_layer(app: &OctantApp, id: crate::app::layers::LayerId) -> Self {
        let Some(layer) = app
            .layers
            .get(id)
            .filter(|_| id != crate::app::layers::LayerId::BASE)
        else {
            return Self::of(app);
        };
        Self {
            plot_type: PlotType::Heatmap,
            volume_algorithm: app.volume_algorithm,
            composite: layer.composite.enabled,
            line_custom_color: false,
            line_all_series: false,
            translucent: layer.color.is_translucent(),
            geographic: false,
        }
    }

    /// The state of the plot on `app`'s canvas.
    pub(crate) fn of(app: &OctantApp) -> Self {
        let base = &app.layers.base;
        Self {
            plot_type: app.effective_canvas_plot_type(),
            volume_algorithm: app.volume_algorithm,
            composite: base.composite.enabled,
            line_custom_color: app.line_use_custom_color,
            line_all_series: app.line_plot_all_series,
            translucent: app.has_color_alpha(),
            geographic: !app.is_ome_dataset() && base.composite.channel_configs.is_empty(),
        }
    }

    pub(crate) fn support(&self) -> OptionSupport {
        OptionSupport {
            color_range: self.color_range(),
            color_mapping: self.color_mapping(),
            nan_color: self.nan_color(),
            opacity: self.opacity(),
            transparency: self.transparency(),
            volume_transparency: self.volume_transparency(),
            coastlines: support_if(self.geographic && self.plot_type.draws_coastlines()),
            // Composites skip the pyramid, but the toggle stays reachable:
            // while it is on, the plot type menu keeps to the heatmap.
            aggregation: support_if(self.plot_type == PlotType::Heatmap),
            camera: support_if(self.plot_type.is_3d()),
        }
    }

    fn indexed_volume(&self) -> bool {
        self.plot_type == PlotType::Volume && self.volume_algorithm == VOLUME_INDEXED_RGBA
    }

    fn color_range(&self) -> Support {
        if self.composite {
            Support::Overridden(COMPOSITE)
        } else if self.indexed_volume() {
            Support::Overridden(INDEXED)
        } else {
            Support::Yes
        }
    }

    fn color_mapping(&self) -> Support {
        match self.color_range() {
            Support::Yes => self.line_override().unwrap_or(Support::Yes),
            other => other,
        }
    }

    /// The line settings that replace the colormap, when the plot is a line.
    fn line_override(&self) -> Option<Support> {
        if self.plot_type != PlotType::Line {
            None
        } else if self.line_custom_color {
            Some(Support::Overridden(CUSTOM_COLOR))
        } else if self.line_all_series {
            Some(Support::Overridden(ALL_SERIES))
        } else {
            None
        }
    }

    fn nan_color(&self) -> Support {
        if self.plot_type == PlotType::Line {
            // Missing values leave gaps in the line.
            Support::No
        } else if self.indexed_volume() {
            Support::Overridden(INDEXED_MISSING)
        } else {
            Support::Yes
        }
    }

    fn opacity(&self) -> Support {
        if self.composite {
            Support::Overridden(COMPOSITE)
        } else if self.plot_type == PlotType::Volume && self.volume_algorithm != VOLUME_DVR {
            Support::Overridden(CLASSIC_OPAQUE)
        } else {
            self.line_override().unwrap_or(Support::Yes)
        }
    }

    fn transparency(&self) -> Support {
        if !matches!(
            self.plot_type,
            PlotType::Sphere | PlotType::Surface | PlotType::PointCloud
        ) {
            Support::No
        } else if self.composite {
            Support::Overridden(COMPOSITE_OPAQUE)
        } else if !self.translucent {
            Support::Overridden(NOT_TRANSLUCENT)
        } else {
            Support::Yes
        }
    }

    fn volume_transparency(&self) -> Support {
        if self.plot_type != PlotType::Volume {
            return Support::No;
        }
        match self.volume_algorithm {
            VOLUME_LABEL_SURFACE => Support::Overridden(LABEL_OPAQUE),
            VOLUME_ADDITIVE_RGBA | VOLUME_INDEXED_RGBA => Support::Overridden(ALWAYS_BLENDS),
            _ => Support::Yes,
        }
    }
}

fn support_if(applies: bool) -> Support {
    if applies { Support::Yes } else { Support::No }
}
