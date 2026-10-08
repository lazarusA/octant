//! What a toast says: its severity, text and optional action.

use std::path::PathBuf;
use std::time::Duration;

use crate::ui::icons::{Icon, IconTone};

/// How serious a toast is; sets its icon, tone and lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Success,
    Warning,
    Error,
}

impl Severity {
    pub fn icon(self) -> Icon {
        match self {
            Self::Info => Icon::Info,
            Self::Success => Icon::Check,
            Self::Warning => Icon::Warning,
            Self::Error => Icon::Cross,
        }
    }

    pub fn tone(self) -> IconTone {
        match self {
            Self::Info => IconTone::Info,
            Self::Success => IconTone::Success,
            Self::Warning => IconTone::Warning,
            Self::Error => IconTone::Error,
        }
    }

    /// How long the toast shows while not hovered.
    pub fn lifetime(self) -> Duration {
        Duration::from_secs(match self {
            Self::Info | Self::Success => 4,
            Self::Warning => 8,
            Self::Error => 12,
        })
    }

    /// Drop order when the stack is full: lower ranks go first.
    pub(super) fn keep_rank(self) -> u8 {
        match self {
            Self::Info | Self::Success => 0,
            Self::Warning => 1,
            Self::Error => 2,
        }
    }

    /// Writes the notice to the log (stderr, or the browser console).
    pub(crate) fn log(self, title: &str, detail: &str) {
        let sep = if detail.is_empty() { "" } else { ": " };
        match self {
            Self::Error => log::error!("{title}{sep}{detail}"),
            Self::Warning => log::warn!("{title}{sep}{detail}"),
            Self::Info | Self::Success => log::info!("{title}{sep}{detail}"),
        }
    }
}

/// A button offered on the toast.
#[derive(Debug, Clone, PartialEq)]
pub enum ToastAction {
    /// Show the file in the system file manager.
    RevealFile(PathBuf),
}

/// A message to show: a short `title` and an optional longer `detail`
/// (empty when there is none).
#[derive(Debug, Clone, PartialEq)]
pub struct Notice {
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    pub action: Option<ToastAction>,
}

impl Notice {
    pub fn new(severity: Severity, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            severity,
            title: title.into(),
            detail: detail.into(),
            action: None,
        }
    }

    pub fn with_action(mut self, action: ToastAction) -> Self {
        self.action = Some(action);
        self
    }
}

/// A notice on screen.
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub notice: Notice,
    /// Times the same notice arrived while this toast showed.
    pub count: u32,
    /// Time left on screen; it does not run while the toast is hovered.
    pub remaining: Duration,
}
