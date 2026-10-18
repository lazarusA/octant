//! Keyboard navigation across the window: one shortcut table (`keymap`),
//! run once per frame before the panels draw (`dispatch`); shared focus
//! handling for widgets that navigate with keys (`focus`, `list_nav`); the
//! regions F6 moves between (`regions`); and the shortcut list (`help`,
//! `labels`).

pub mod dispatch;
pub mod focus;
pub mod help;
#[cfg(test)]
mod help_tests;
pub mod keymap;
pub mod labels;
pub mod list_nav;
#[cfg(test)]
mod panel_tests;
#[cfg(test)]
mod playback_tests;
pub mod regions;
#[cfg(test)]
mod tests;

pub use dispatch::dispatch;
