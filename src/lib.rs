#![cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]

pub mod app;
pub mod catalog;
pub mod data;
pub mod export;
pub mod plots;
pub mod ui;
pub mod utils;
