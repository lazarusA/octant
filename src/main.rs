mod app;
pub mod catalog;
pub mod data;
pub mod plots;
pub mod ui;
pub mod utils;

use app::OctantApp;

// NATIVE RUNTIME MAIN WINDOW TRIGGER
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // env_logger::Builder::from_env(
    //     env_logger::Env::default().default_filter_or("info,octant=debug,wgpu=warn"),
    // )
    // .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1150.0, 720.0])
            .with_title("Octant: N-dimensional Data Explorer"),
        depth_buffer: 32,
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
    let _ = console_log::init_with_level(log::Level::Debug);

    let web_options = eframe::WebOptions {
        depth_buffer: 32,
        ..Default::default()
    };
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
