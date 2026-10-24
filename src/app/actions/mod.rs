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
pub mod phrases;
pub mod resolve;
#[cfg(test)]
mod tests;
mod text;

use serde::{Deserialize, Serialize};

use super::OctantApp;
use crate::app::layers::{LayerId, LevelChoice};
use crate::plots::PlotType;
use phrases::Parsed;
use resolve::{Resolution, Vocabulary};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AppAction {
    /// Stages a variable of the active dataset, as clicking it in the
    /// Variables panel does.
    SelectVariable {
        path: String,
    },
    /// Plots the staged selection (Plot Data), as `plot_type` when given.
    Plot {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        plot_type: Option<String>,
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

/// What an action did.
#[derive(Debug, Clone, PartialEq)]
pub enum Done {
    Selected(String),
    Plotted {
        variable: String,
        plot_type: PlotType,
    },
    Playing,
    Paused,
    Stepped(i32),
    Colormap(String),
    ViewReset,
}

/// What a name named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameKind {
    Variable,
    PlotType,
    Colormap,
}

/// Why an action did nothing.
#[derive(Debug, Clone, PartialEq)]
pub enum Failure {
    NoDataset,
    NotFound {
        kind: NameKind,
        text: String,
        closest: Vec<String>,
    },
    Ambiguous {
        kind: NameKind,
        text: String,
        matches: Vec<String>,
    },
    /// The staged variable cannot be drawn as this plot type.
    Blocked {
        plot_type: PlotType,
        reason: String,
    },
    NothingToAnimate,
    NotUnderstood(String),
}

pub type Outcome = Result<Done, Failure>;

/// `resolution` of `text` as a value or the failure it answers.
fn resolved<T>(resolution: Resolution<T>, kind: NameKind, text: &str) -> Result<T, Failure> {
    let text = text.to_string();
    match resolution {
        Resolution::Resolved(value) => Ok(value),
        Resolution::Ambiguous(matches) => Err(Failure::Ambiguous {
            kind,
            text,
            matches,
        }),
        Resolution::NotFound(closest) => Err(Failure::NotFound {
            kind,
            text,
            closest,
        }),
    }
}

impl OctantApp {
    /// What the names of actions can find right now.
    pub fn vocabulary(&self) -> Vocabulary {
        Vocabulary::new(self.selected.metadata.as_ref())
    }

    /// Runs `action` against the app as it is now.
    pub fn dispatch(&mut self, action: AppAction) -> Outcome {
        match action {
            AppAction::SelectVariable { path } => self.select_by_name(&path),
            AppAction::Plot { plot_type } => self.plot_as(plot_type.as_deref()),
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

    fn select_by_name(&mut self, path: &str) -> Outcome {
        if self.selected.metadata.is_none() {
            return Err(Failure::NoDataset);
        }
        let idx = resolved(self.vocabulary().variable(path), NameKind::Variable, path)?;
        // Selecting the staged variable again would reset its dimensions.
        if idx != self.selected.variable_idx {
            crate::ui::variables_overlay::apply_selection(self, idx, LevelChoice::Auto);
        }
        Ok(Done::Selected(self.staged_name()))
    }

    fn plot_as(&mut self, plot_type: Option<&str>) -> Outcome {
        if self.selected.metadata.is_none() {
            return Err(Failure::NoDataset);
        }
        if let Some(text) = plot_type {
            let plot_type = resolved(self.vocabulary().plot_type(text), NameKind::PlotType, text)?;
            let options = crate::ui::plot_type::options::plot_options(self);
            if let Some(blocked) = options
                .iter()
                .find(|o| o.plot_type == plot_type)
                .and_then(|o| o.blocked)
            {
                let mut buf = [0; 256];
                let reason =
                    crate::ui::plot_type::options::blocked_text(&mut buf, blocked).to_string();
                return Err(Failure::Blocked { plot_type, reason });
            }
            self.switch_plot_type(plot_type);
        }
        self.plot_from_panel();
        Ok(Done::Plotted {
            variable: self.staged_name(),
            plot_type: self.selected.plot_type,
        })
    }

    fn staged_name(&self) -> String {
        self.selected_variable_info()
            .map(|v| v.display_name().to_string())
            .unwrap_or_default()
    }

    /// Whether the plot has an animated dimension to play or step along.
    fn base_animates(&self) -> Result<(), Failure> {
        match self.layers.base.selection().animated_dim {
            Some(_) if self.plotted().metadata.is_some() => Ok(()),
            _ => Err(Failure::NothingToAnimate),
        }
    }

    /// Runs what `utterance` says, command by command, stopping at the first
    /// that fails; what the last did, or why one failed.
    pub fn run_utterance(&mut self, utterance: &str) -> Vec<Outcome> {
        let vocab = self.vocabulary();
        let mut outcomes = Vec::new();
        for parsed in phrases::parse(utterance, &vocab, crate::i18n::language()) {
            let outcome = match parsed {
                Parsed::Action(action) => self.dispatch(action),
                Parsed::NotUnderstood(text) => Err(Failure::NotUnderstood(text)),
            };
            let failed = outcome.is_err();
            outcomes.push(outcome);
            if failed {
                break;
            }
        }
        outcomes
    }

    /// Runs the voice bar's finished utterances: what they did is shown in
    /// the bar, why one failed as a toast.
    pub(crate) fn run_voice_commands(&mut self) {
        for utterance in self.voice.take_utterances() {
            for outcome in self.run_utterance(&utterance) {
                match outcome {
                    Ok(done) => self.voice.set_last_action(answer::done_text(&done)),
                    Err(failure) => self.push_notice(answer::failure_notice(&failure)),
                }
            }
        }
    }
}
