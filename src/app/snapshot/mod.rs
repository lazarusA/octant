//! `octant --snapshot <file.png> <link>` (desktop only): opens the link,
//! waits until the plot has loaded and settled, saves the canvas and quits.
//! `request.rs` reads the command line, `drive.rs` runs the snapshot frame
//! by frame. A failure prints its reason on stderr and the process exits
//! with 1 (`exit_unless_saved`), so scripts can tell bad links apart.

mod drive;
mod request;
#[cfg(test)]
mod request_tests;

use std::path::Path;
use std::time::SystemTime;

use web_time::Instant;

pub use request::{Request, Startup};

/// The window's inner size in points while capturing.
pub const WINDOW: [f32; 2] = [1600.0, 900.0];

use super::OctantApp;

/// A snapshot under way (`OctantApp::snapshot`).
#[derive(Debug)]
pub struct Snapshot {
    request: Request,
    start: Instant,
    /// Since when the plot has been ready; `None` while it is not.
    ready_since: Option<Instant>,
    /// Since when nothing has loaded or waited, yet no plot shows.
    stalled_since: Option<Instant>,
    /// The window was told to close.
    closing: bool,
    /// Times the window was resized to the image's shape (`shape_window`).
    reshaped: u8,
    /// The canvas's pixels and the window's size last frame.
    measured: Option<([u32; 2], egui::Vec2)>,
    /// The export was asked for; the window closes once it is written.
    requested: bool,
    /// Toasts with a lower id were already read.
    next_toast: u64,
}

impl OctantApp {
    /// Opens the command line's source; with `--snapshot`, captures it.
    pub fn start(&mut self, startup: Startup) {
        if let Some(source) = &startup.source {
            self.submit_or_activate_source(source, None);
        }
        if let Some(request) = &startup.snapshot {
            self.layout.show_axes = request.axes;
        }
        self.snapshot = startup.snapshot.map(|request| Snapshot {
            request,
            start: Instant::now(),
            ready_since: None,
            stalled_since: None,
            closing: false,
            reshaped: 0,
            measured: None,
            requested: false,
            next_toast: 0,
        });
    }
}

/// After the window closed: exits with 1 unless `path` was written since
/// `started` (the window closed by hand, a failure, an export error).
pub fn exit_unless_saved(path: Option<&Path>, started: SystemTime) {
    let Some(path) = path else {
        return;
    };
    let modified = std::fs::metadata(path).and_then(|m| m.modified());
    if !modified.is_ok_and(|m| m >= started) {
        eprintln!("octant: no snapshot saved to {}", path.display());
        std::process::exit(1);
    }
}
