//! Commands waiting their turn. An utterance becomes commands, each read
//! against the app as it is when its turn comes, so the commands after
//! "open" wait until the dataset has loaded and find its names; a dataset
//! that fails to load drops them (its failure is already a toast), as does
//! any command that fails. New utterances queue after those waiting.

use std::collections::VecDeque;

use super::answer;
use super::phrases::{self, Parsed};
use super::text::Segment;
use super::{Failure, Outcome};
use crate::app::OctantApp;
use crate::i18n::Language;

#[derive(Default)]
pub struct CommandQueue {
    /// Commands not yet run, each with the language it is read in.
    waiting: VecDeque<(Language, Segment)>,
    /// The store a command opened, until it settles.
    opening: Option<String>,
}

impl CommandQueue {
    pub(super) fn await_dataset(&mut self, store: &str) {
        self.opening = Some(store.to_string());
    }

    fn clear(&mut self) {
        self.waiting.clear();
        self.opening = None;
    }
}

impl OctantApp {
    /// Queues what `utterance` says and runs what can run now: what each
    /// did, or why one failed.
    pub fn run_utterance(&mut self, utterance: &str) -> Vec<Outcome> {
        let commands = phrases::commands(utterance, crate::i18n::language());
        self.commands.waiting.extend(commands);
        self.resume_commands()
    }

    /// Runs the waiting commands until they are done, one fails, or one
    /// opened a dataset still loading.
    pub fn resume_commands(&mut self) -> Vec<Outcome> {
        let mut outcomes = Vec::new();
        while self.opened_dataset_settled() {
            let Some((lang, segment)) = self.commands.waiting.pop_front() else {
                break;
            };
            let vocab = self.vocabulary();
            for parsed in phrases::parse_command(lang, &segment, &vocab) {
                let outcome = match parsed {
                    Parsed::Action(action) => self.dispatch(action),
                    Parsed::NotUnderstood(text) => Err(Failure::NotUnderstood(text)),
                };
                let failed = outcome.is_err();
                outcomes.push(outcome);
                if failed {
                    self.commands.clear();
                    return outcomes;
                }
            }
        }
        outcomes
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
    /// a dataset: what they did is shown in the bar, why one failed as a
    /// toast.
    pub(crate) fn run_voice_commands(&mut self) {
        let mut outcomes = self.resume_commands();
        for utterance in self.voice.take_utterances() {
            outcomes.extend(self.run_utterance(&utterance));
        }
        for outcome in outcomes {
            match outcome {
                Ok(done) => self.voice.set_last_action(answer::done_text(&done)),
                Err(failure) => {
                    if let Some(notice) = answer::failure_notice(&failure) {
                        self.push_notice(notice);
                    }
                }
            }
        }
    }
}
