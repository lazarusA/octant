//! The action layer: what a spoken or typed command (and later a language
//! model's tool call) does to the app, one path for every input.
//!
//! Actions are language-neutral and name what they act on as the dataset
//! does (a variable by its path, name or long name), resolved when they run
//! against the dataset loaded then (`resolve`), whatever its format. They
//! keep exploring apart from the plot, as the UI does: selecting stages a
//! variable, `Plot` plots it, playback, steps and colors act on the plot.
//! `dispatch` calls the methods the UI calls, never copies of them.

mod answer;
mod ask;
#[cfg(test)]
mod ask_tests;
mod axes;
mod camera;
mod candidates;
mod coord_match;
#[cfg(test)]
mod dates_tests;
mod dims;
#[cfg(test)]
mod dims_tests;
mod dispatch;
mod layers;
#[cfg(test)]
mod layers_tests;
mod open;
#[cfg(test)]
mod open_tests;
mod outcome;
mod phrases;
mod plate;
mod queue;
pub mod resolve;
#[cfg(test)]
mod retry_tests;
pub mod schema;
#[cfg(test)]
mod schema_tests;
pub mod source_names;
mod staging;
mod style;
#[cfg(test)]
mod tests;
mod text;
mod view;

use serde::{Deserialize, Serialize};

pub(crate) use candidates::plot_type_id;
pub(crate) use coord_match::value_text;
use outcome::resolved;
pub use outcome::{Done, Failure, NameKind, Outcome};
pub use queue::{CommandQueue, Origin};
pub(crate) use style::{CURVES, hex_rgba};
pub use style::{ChannelSetting, ColorSetting, CompositeSetting};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AppAction {
    /// Stages a variable of the active dataset, as clicking it in the
    /// Variables panel does.
    SelectVariable {
        path: String,
    },
    /// Opens a dataset: a landing sample, catalog entry or open dataset by
    /// name, or an address; the commands after it wait until it loads.
    OpenSource {
        source: String,
    },
    /// Shows the dataset catalog.
    OpenCatalog,
    /// What the active dataset (or its group `group`) holds.
    DescribeDataset {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        group: Option<String>,
    },
    /// Searches the active dataset's variables (the Variables panel's search).
    Search {
        text: String,
    },
    /// What a variable is (the staged one without `path`): its long name,
    /// units and dimensions with their coordinate ranges.
    DescribeVariable {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    /// What is plotted (what the dataset holds when nothing is).
    DescribeView,
    /// Reads the staged multiscale variable at level `level` (0 is the
    /// finest), or at the level that fits the screen without one; with
    /// `auto`, reads at least `level`'s detail and still refines as the view
    /// needs. A variable without levels stays as it is. Plots it again when
    /// it is the one shown.
    SetLevel {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        level: Option<usize>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        auto: bool,
    },
    /// Plots the staged selection (Plot Data), as `plot_type` when given.
    Plot {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        plot_type: Option<String>,
    },
    /// Moves dimension `dim` of the staged selection (its animated one: the
    /// plot's step) to `value`: an ISO date, a number in the coordinate's
    /// units, a coordinate label, or without coordinates a 1-based position.
    /// Without `dim`, the time dimension.
    SetDim {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dim: Option<String>,
        value: String,
    },
    /// Dimension `dim` reads `from..=to`, values as in `SetDim`.
    SetRange {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dim: Option<String>,
        from: String,
        to: String,
    },
    /// Dimension `dim`'s role, as its Dimensions panel selects set it.
    SetRole {
        dim: String,
        role: DimRole,
    },
    Play,
    Pause,
    /// Moves the plot `by` steps along its animated dimension.
    Step {
        by: i32,
    },
    /// Adds the staged variable as an overlay of the plotted heatmap (Plot
    /// Data with Add Overlay on).
    AddOverlay,
    /// Shows or hides a layer, named by its variable.
    SetLayerVisible {
        layer: String,
        visible: bool,
    },
    /// Removes an overlay, named by its variable.
    RemoveOverlay {
        layer: String,
    },
    /// A layer's colormap (the base layer's without `layer`), by name or
    /// `family:name` key.
    SetColormap {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
    },
    /// A layer's color range, locked as the colorbar's inputs lock it.
    SetColorRange {
        min: f32,
        max: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layer: Option<String>,
    },
    /// Opens the export dialog (on `format`, an extension: png, jpg, webp,
    /// svg, pdf),
    /// whose own button exports.
    Export {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<String>,
    },
    /// Picks a well of the open plate (the picked one without `well`) and
    /// opens its field number `field` (1-based, as its button shows).
    OpenPlateWell {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        well: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        field: Option<usize>,
    },
    /// The plotted variable's color composite: on or off, its R/G/B bands
    /// or the channels it draws.
    SetComposite {
        #[serde(flatten)]
        composite: CompositeSetting,
    },
    /// The plot's color scale, opacity, reversal and NaN and clip colors;
    /// those left out are kept.
    SetColorStyle {
        #[serde(flatten)]
        style: ColorSetting,
    },
    ResetView,
    /// Zooms the heatmap to show dimension x from `x_from` to `x_to` and y
    /// from `y_from` to `y_to` (values as in `SetRange`); the plot then
    /// reads the finest level the view needs.
    SetView {
        x_from: String,
        x_to: String,
        y_from: String,
        y_to: String,
    },
    /// Turns the 3D camera to `pitch` and `yaw` (degrees) at `distance`
    /// from what it looks at, `target` (the center without one).
    SetCamera {
        pitch: f32,
        yaw: f32,
        distance: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<[f32; 3]>,
    },
}

/// A dimension's role: a plot axis, animated, or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DimRole {
    X,
    Y,
    Z,
    Animate,
    None,
}
