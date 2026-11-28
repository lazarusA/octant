#![cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]

pub mod app;
pub mod data;
pub mod export;
pub mod gallery;
pub mod i18n;
pub mod plots;
pub mod ui;
pub mod utils;
pub mod voice;
