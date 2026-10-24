//! Commands waiting their turn. An utterance becomes commands, each read
//! against the app as it is when its turn comes, so the commands after
//! "open" wait until the dataset has loaded and find its names; a dataset
//! that fails to load drops them (its failure is already a toast), as does
//! any action that fails. An action waiting for something being read
//! (coordinates, a plate's wells) stays first until it arrives. New utterances queue after those waiting.

use std::collections::VecDeque;

use super::answer;
use super::phrases::{self, Parsed};
use super::text::Segment;
use super::{Done, Failure, Outcome};
use crate::app::OctantApp;
use crate::i18n::Language;
use crate::ui::panel_layout::Panel;
use crate::ui::toast::Severity;

/// A command not yet read, or what it was read into, not yet run.
enum Item {
    Command(Language, Segment),
    Read(Parsed),
}

#[derive(Default)]
pub struct CommandQueue {
    waiting: VecDeque<Item>,
    /// The store a command opened, until it settles.
    opening: Option<String>,
    /// Whether the first action waits for something being read.
    reading: bool,
}

impl CommandQueue {
    pub(super) fn await_dataset(&mut self, store: &str) {
        self.opening = Some(store.to_string());
    }

    fn clear(&mut self) {
        self.waiting.clear();
        self.opening = None;
        self.reading = false;
    }
}

impl OctantApp {
    /// Queues what `utterance` says and runs what can run now: what each
    /// did, or why one failed.
    pub fn run_utterance(&mut self, utterance: &str) -> Vec<Outcome> {
        let commands = phrases::commands(utterance, crate::i18n::language());
        let items = commands.into_iter().map(|(lang, s)| Item::Command(lang, s));
        self.commands.waiting.extend(items);
        self.resume_commands()
    }

    /// Runs the waiting commands until they are done, one fails, or one
    /// waits for a dataset or coordinates still loading.
    pub fn resume_commands(&mut self) -> Vec<Outcome> {
        let mut outcomes = Vec::new();
        while self.opened_dataset_settled() && self.reads_settled() {
            let action = match self.commands.waiting.pop_front() {
                None => break,
                Some(Item::Read(Parsed::Action(action))) => action,
                Some(Item::Read(Parsed::NotUnderstood(text))) => {
                    outcomes.push(Err(Failure::NotUnderstood(text)));
                    self.commands.clear();
                    break;
                }
                Some(Item::Command(lang, segment)) => {
                    let vocab = self.vocabulary();
                    let parsed = phrases::parse_command(lang, &segment, &vocab);
                    for read in parsed.into_iter().rev() {
                        self.commands.waiting.push_front(Item::Read(read));
                    }
                    continue;
                }
            };
            match self.dispatch(action.clone()) {
                Err(Failure::Reading) => {
                    let waiting = Item::Read(Parsed::Action(action));
                    self.commands.waiting.push_front(waiting);
                    self.commands.reading = true;
                }
                Err(failure) => {
                    outcomes.push(Err(failure));
                    self.commands.clear();
                    break;
                }
                done => outcomes.push(done),
            }
        }
        outcomes
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
        if let Some(store) = self.commands.opening.take()
            && (self.selected.metadata.is_none() || self.selected.store_target != store)
        {
            self.commands.clear();
        }
        true
    }

    /// Runs the voice bar's finished utterances and the commands waiting on
    /// a dataset: what they did is shown in the bar (with an answer's
    /// detail as a toast, and the Variables panel opened for what the
    /// dataset holds), why one failed as a toast.
    pub(crate) fn run_voice_commands(&mut self, ctx: &egui::Context) {
        let mut outcomes = self.resume_commands();
        for utterance in self.voice.take_utterances() {
            outcomes.extend(self.run_utterance(&utterance));
        }
        for outcome in outcomes {
            match outcome {
                Ok(done) => self.show_done(ctx, &done),
                Err(failure) => {
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
