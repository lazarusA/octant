//! Commands waiting their turn. An utterance becomes commands, each read
//! against the app as it is when its turn comes, so the commands after
//! "open" wait until the dataset has loaded and find its names; a dataset
//! that fails to load drops them (its failure is already a toast), as does
//! any action that fails. An action waiting for something being read
//! (coordinates, a plate's wells) stays first until it arrives. New utterances queue after those waiting.
//! The queue runs every frame whether or not the voice bar is open; each
//! item keeps its `Origin`, which decides what cancels it and how it reports.

use std::collections::VecDeque;

use super::answer;
use super::phrases::{self, Parsed};
use super::resolve::Vocabulary;
use super::text::Segment;
use super::{AppAction, Done, Failure, Outcome};
use crate::app::OctantApp;
use crate::i18n::Language;
use crate::ui::panel_layout::Panel;
use crate::ui::toast::Severity;

/// A command not yet read, or what it was read into, not yet run, or an
/// action that waited for a read and runs again now that it settled.
enum Item {
    Command(Language, Segment),
    Read(Parsed),
    Retry(AppAction),
    /// An action built elsewhere (a share link), never read as words.
    Action(AppAction),
}

/// Where queued commands came from: the voice bar (said or typed), dropped
/// when the bar closes, or a share link, replaced by the next link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Origin {
    /// Also an action dispatched outside the queue (tests).
    #[default]
    Voice,
    Link,
}

#[derive(Default)]
pub struct CommandQueue {
    waiting: VecDeque<(Origin, Item)>,
    /// The store a command opened and where that command came from, until
    /// it settles: a store that fails drops that origin's commands only.
    opening: Option<(Origin, String)>,
    /// Where the action running came from.
    running: Origin,
    /// Whether a `Plot` the queue ran still waits for its block: the
    /// commands after it wait too, or its arrival would replace their colors.
    plotting: bool,
    /// Whether the first action waits for something being read.
    reading: bool,
    /// Whether the action running waited for a read: it asks for nothing
    /// again, so a read that failed never loops.
    retrying: bool,
    /// The names commands find, and what they were read from.
    pub(super) vocab: Option<(VocabKey, Vocabulary)>,
}

/// What a vocabulary was built from: the active dataset (its store and
/// metadata), the open datasets, the open plate's wells and the colormaps.
pub(super) type VocabKey = (String, u64, usize, usize, u64);

impl CommandQueue {
    pub(super) fn retrying(&self) -> bool {
        self.retrying
    }

    pub(super) fn await_dataset(&mut self, store: &str) {
        self.opening = Some((self.running, store.to_string()));
    }

    /// Drops the commands waiting from `origin` (one failed, the voice bar
    /// closed, a new link), and the dataset they waited for.
    fn drop_origin(&mut self, origin: Origin) {
        self.waiting.retain(|(o, _)| *o != origin);
        if self.opening.as_ref().is_some_and(|(o, _)| *o == origin) {
            self.opening = None;
        }
        if self.waiting.is_empty() {
            self.reading = false;
        }
    }
}

impl OctantApp {
    /// Queues what `utterance` says and runs what can run now: what each
    /// did, or why one failed (tests; the app queues through `run_commands`).
    #[cfg(test)]
    pub fn run_utterance(&mut self, utterance: &str) -> Vec<Outcome> {
        self.queue_utterance(utterance);
        self.resume_commands()
    }

    fn queue_utterance(&mut self, utterance: &str) {
        let commands = phrases::commands(utterance, crate::i18n::language());
        let items = commands
            .into_iter()
            .map(|(lang, s)| (Origin::Voice, Item::Command(lang, s)));
        self.commands.waiting.extend(items);
    }

    /// Queues `actions` from `origin`, after what waits; a link replaces the
    /// link commands still waiting. They run from the next `run_commands`.
    pub fn queue_actions(&mut self, actions: Vec<AppAction>, origin: Origin) {
        if origin == Origin::Link {
            self.commands.drop_origin(Origin::Link);
        }
        let items = actions.into_iter().map(|a| (origin, Item::Action(a)));
        self.commands.waiting.extend(items);
    }

    /// Runs the waiting commands until they are done, one fails, or one
    /// waits for a dataset or coordinates still loading (tests).
    #[cfg(test)]
    pub fn resume_commands(&mut self) -> Vec<Outcome> {
        self.resume().into_iter().map(|(_, o)| o).collect()
    }

    fn resume(&mut self) -> Vec<(Origin, Outcome)> {
        let mut outcomes = Vec::new();
        while self.opened_dataset_settled()
            && !self.commands.waiting.is_empty()
            && self.reads_settled()
            && self.plot_settled()
        {
            let Some((origin, item)) = self.commands.waiting.pop_front() else {
                break;
            };
            let (action, retry) = match item {
                Item::Read(Parsed::Action(action)) | Item::Action(action) => (action, false),
                Item::Retry(action) => (action, true),
                Item::Read(Parsed::NotUnderstood(text)) => {
                    outcomes.push((origin, Err(Failure::NotUnderstood(text))));
                    self.commands.drop_origin(origin);
                    break;
                }
                Item::Command(lang, segment) => {
                    let parsed = phrases::parse_command(lang, &segment, self.vocabulary());
                    for read in parsed.into_iter().rev() {
                        self.commands.waiting.push_front((origin, Item::Read(read)));
                    }
                    continue;
                }
            };
            self.commands.retrying = retry;
            self.commands.running = origin;
            let plots = matches!(action, AppAction::Plot { .. });
            let outcome = self.dispatch(action.clone());
            self.commands.running = Origin::default();
            self.commands.retrying = false;
            self.commands.plotting |= plots && outcome.is_ok();
            match outcome {
                Err(Failure::Reading) => {
                    self.commands
                        .waiting
                        .push_front((origin, Item::Retry(action)));
                    self.commands.reading = true;
                }
                Err(failure) => {
                    outcomes.push((origin, Err(failure)));
                    self.commands.drop_origin(origin);
                    break;
                }
                done => outcomes.push((origin, done)),
            }
        }
        // Nothing left waits on a plot: one started by hand holds nothing.
        self.commands.plotting &= !self.commands.waiting.is_empty();
        outcomes
    }

    /// Whether no `Plot` the queue ran still waits for its block.
    fn plot_settled(&mut self) -> bool {
        self.commands.plotting = self.commands.plotting && self.new_plot_pending();
        !self.commands.plotting
    }

    /// Whether nothing an action waits for (coordinates, a plate's wells) is
    /// still being read.
    fn reads_settled(&mut self) -> bool {
        let busy = self.coordinate_loader.pending_count() > 0
            || self.plate.as_ref().is_some_and(|p| p.loading());
        let reading = self.commands.reading && busy;
        self.commands.reading = reading;
        !reading
    }

    /// Whether no dataset a command opened is still loading; one that did
    /// not become the active dataset drops the commands after it.
    fn opened_dataset_settled(&mut self) -> bool {
        if self.metadata_rx.is_some() {
            return false;
        }
        if let Some((origin, store)) = self.commands.opening.take()
            && (self.selected.metadata.is_none() || self.selected.store_target != store)
        {
            self.commands.drop_origin(origin);
        }
        true
    }

    /// Runs the commands waiting, every frame, and the voice bar's finished
    /// utterances while it is open (closing it drops the voice commands
    /// waiting). What voice commands did is shown in the bar (with an
    /// answer's detail as a toast, and the Variables panel opened for what
    /// the dataset holds); link commands show only their failures; why one
    /// failed is a toast.
    pub(crate) fn run_commands(&mut self, ctx: &egui::Context) {
        if self.voice.is_open() {
            for utterance in self.voice.take_utterances() {
                self.queue_utterance(&utterance);
            }
        } else {
            // What waited for the bar never runs: it would meet another state.
            self.commands.drop_origin(Origin::Voice);
        }
        for (origin, outcome) in self.resume() {
            match (origin, outcome) {
                (Origin::Link, Ok(_)) => {}
                (Origin::Voice, Ok(done)) => self.show_done(ctx, &done),
                (Origin::Voice, Err(Failure::NotUnderstood(text))) => {
                    self.voice
                        .set_last_action(answer::not_understood_line(&text));
                }
                (_, Err(failure)) => {
                    if let Some(notice) = answer::failure_notice(&failure) {
                        self.push_notice(notice);
                    }
                }
            }
        }
    }

    fn show_done(&mut self, ctx: &egui::Context, done: &Done) {
        let line = answer::done_text(done);
        if let Some(detail) = answer::done_detail(done) {
            self.notify(Severity::Info, &line, detail);
        }
        if matches!(done, Done::DatasetShown { .. } | Done::Searching { .. }) {
            self.set_panel_open(ctx, Panel::Variables, true);
        }
        self.voice.set_last_action(line);
    }
}

#[cfg(test)]
#[path = "queue_tests.rs"]
mod tests;
