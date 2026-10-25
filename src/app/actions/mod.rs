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
mod candidates;
mod coord_match;
#[cfg(test)]
mod dates_tests;
mod dims;
#[cfg(test)]
mod dims_tests;
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
#[cfg(test)]
mod tests;
mod text;

use serde::{Deserialize, Serialize};

use super::OctantApp;
use outcome::resolved;
pub use outcome::{Done, Failure, NameKind, Outcome};
pub use queue::CommandQueue;

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
    ResetView,
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

impl OctantApp {
    /// Runs `action` against the app as it is now.
    pub fn dispatch(&mut self, action: AppAction) -> Outcome {
        match action {
            AppAction::OpenSource { source } => self.open_by_name(&source),
            AppAction::OpenCatalog => {
                self.open_sample(crate::app::sources::CATALOG_SAMPLE);
                Ok(Done::CatalogOpened)
            }
            AppAction::DescribeDataset { group } => self.describe_dataset(group.as_deref()),
            AppAction::Search { text } => self.search_variables(&text),
            AppAction::DescribeVariable { path } => self.describe_variable(path.as_deref()),
            AppAction::DescribeView => self.describe_view(),
            AppAction::SelectVariable { path } => self.select_by_name(&path),
            AppAction::Plot { plot_type } => self.plot_as(plot_type.as_deref()),
            AppAction::SetDim { dim, value } => self.set_dim(dim.as_deref(), &value),
            AppAction::SetRange { dim, from, to } => self.set_dim_range(dim.as_deref(), &from, &to),
            AppAction::SetRole { dim, role } => self.set_dim_role(&dim, role),
            AppAction::Play => self.play(true),
            AppAction::Pause => self.play(false),
            AppAction::Step { by } => self.step_by(by),
            AppAction::AddOverlay => self.add_staged_overlay(),
            AppAction::SetLayerVisible { layer, visible } => self.show_layer(&layer, visible),
            AppAction::RemoveOverlay { layer } => self.remove_layer(&layer),
            AppAction::SetColorRange { min, max, layer } => {
                self.set_color_range(layer.as_deref(), min, max)
            }
            AppAction::Export { format } => self.open_export(format.as_deref()),
            AppAction::OpenPlateWell { well, field } => {
                self.open_plate_well(well.as_deref(), field)
            }
            AppAction::SetColormap { name, layer } => self.set_colormap(&name, layer.as_deref()),
            AppAction::ResetView => {
                let plot_type = self.effective_canvas_plot_type();
                crate::app::controllers::controller_for(plot_type).reset_view(&mut self.nav);
                Ok(Done::ViewReset)
            }
        }
    }
}
