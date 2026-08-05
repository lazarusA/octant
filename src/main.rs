mod app;
pub mod cache;
pub mod catalog;
pub mod data;
pub mod plots;
mod stores;
pub mod ui;
pub mod utils;

use app::OctantApp;

// NATIVE RUNTIME MAIN WINDOW TRIGGER
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1150.0, 720.0])
            .with_title("Octant — Interactive Visualization of n-dimensional datasets"),
        ..Default::default()
    };

    eframe::run_native(
        "Octant",
        native_options,
        Box::new(|cc| Ok(Box::new(OctantApp::new(cc)))),
    )
}

// BROWSER INTERFACE MOUNT BRIDGE
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main_web() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "octant_canvas_anchor",
                web_options,
                Box::new(|cc| Ok(Box::new(OctantApp::new(cc)))),
            )
            .await
            .expect("Failed to bind GPU context execution channel target element");
    });
}
