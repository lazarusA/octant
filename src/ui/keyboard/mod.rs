//! Keyboard navigation across the window: one shortcut table (`keymap`),
//! run once per frame before the panels draw (`dispatch`); shared focus
//! handling for widgets that navigate with keys (`focus`, `list_nav`) and
//! for windows opening and closing (`windows`); the
//! regions F6 moves between (`regions`); and the shortcut list (`help`,
//! `labels`).

mod bindings;
pub mod dispatch;
pub mod focus;
#[cfg(test)]
mod goto_tests;
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
#[cfg(test)]
mod view_tests;
#[cfg(test)]
mod walk_tests;
pub mod windows;

pub use dispatch::dispatch;
