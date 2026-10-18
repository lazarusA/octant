//! Keyboard navigation across the window: one shortcut table (`keymap`),
//! run once per frame before the panels draw (`dispatch`); shared focus
//! handling for widgets that navigate with keys (`focus`); and the regions
//! F6 moves between (`regions`).

pub mod dispatch;
pub mod focus;
pub mod keymap;
#[cfg(test)]
mod panel_tests;
#[cfg(test)]
mod playback_tests;
pub mod regions;
#[cfg(test)]
mod tests;

pub use dispatch::dispatch;
