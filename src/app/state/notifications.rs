//! User-facing notices: shown as canvas toasts and logged.

use super::app_state::OctantApp;
use crate::ui::toast::{Notice, Severity};

impl OctantApp {
    /// Shows a toast with `title` and `detail` (may be empty), and logs it.
    pub fn notify(&mut self, severity: Severity, title: &str, detail: &str) {
        self.push_notice(Notice::new(severity, title, detail));
    }

    /// Shows and logs a prepared notice (e.g. one carrying an action).
    pub fn push_notice(&mut self, notice: Notice) {
        notice.severity.log(&notice.title, &notice.detail);
        self.toasts.push(notice);
    }

    /// Moves the notices reported outside the app (`toast::report`) onto the canvas.
    pub(crate) fn drain_reported_notices(&mut self) {
        for notice in crate::ui::toast::drain_reports() {
            self.toasts.push(notice);
        }
    }
}
