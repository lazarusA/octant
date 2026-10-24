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
mod dims;
#[cfg(test)]
mod dims_tests;
mod open;
#[cfg(test)]
mod open_tests;
mod outcome;
mod phrases;
mod queue;
pub mod resolve;
pub mod source_names;
mod staging;
#[cfg(test)]
mod tests;
mod text;

use serde::{Deserialize, Serialize};

use super::OctantApp;
use crate::app::layers::LayerId;
use outcome::resolved;
pub use outcome::{Done, Failure, NameKind, Outcome};
pub use queue::CommandQueue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// The plot's colormap, by name or `family:name` key.
    SetColormap {
        name: String,
    },
    ResetView,
}

/// A dimension's role: a plot axis, animated, or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
            AppAction::Play | AppAction::Pause => {
                let play = action == AppAction::Play;
                self.base_animates()?;
                if self.playback.is_playing != play {
                    self.toggle_playback();
                }
                Ok(if play { Done::Playing } else { Done::Paused })
            }
            AppAction::Step { by } => {
                self.base_animates()?;
                for _ in 0..by.unsigned_abs() {
                    if by > 0 {
                        self.step_next()
                    } else {
                        self.step_prev()
                    }
                }
                Ok(Done::Stepped(by))
            }
            AppAction::SetColormap { name } => {
                let id = resolved(self.vocabulary().colormap(&name), NameKind::Colormap, &name)?;
                crate::ui::colormap::select_layer_colormap(self, LayerId::BASE, id);
                let key = crate::utils::colormap::registry::key_of(id);
                Ok(Done::Colormap(key))
            }
            AppAction::ResetView => {
                let plot_type = self.effective_canvas_plot_type();
                crate::app::controllers::controller_for(plot_type).reset_view(&mut self.nav);
                Ok(Done::ViewReset)
            }
        }
    }
}
