#![cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]

mod app;
pub mod catalog;
pub mod data;
pub mod export;
pub mod plots;
pub mod ui;
pub mod utils;

use app::OctantApp;

// NATIVE RUNTIME MAIN WINDOW TRIGGER
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // let _ = env_logger::Builder::from_env(
    //     env_logger::Env::default().default_filter_or("info,octant=debug,wgpu=warn,zarrs=warn"),
    // )
    // .try_init();

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
fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    let web_options = eframe::WebOptions {
        depth_buffer: 32,
        ..Default::default()
    };
    wasm_bindgen_futures::spawn_local(async {
        use wasm_bindgen::JsCast;
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");
        let canvas = document
            .get_element_by_id("octant_canvas_anchor")
            .expect("Canvas element not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element is not a canvas");

        let runner = eframe::WebRunner::new();
        if let Err(e) = runner
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(OctantApp::new(cc)))),
            )
            .await
        {
            log::error!("Failed to start Octant web runner: {e:?}");
        }
    });
}
