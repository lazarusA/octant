#![cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]

mod app;
pub mod data;
pub mod export;
pub mod gallery;
pub mod i18n;
pub mod plots;
pub mod ui;
pub mod utils;
pub mod voice;

use app::OctantApp;

// NATIVE RUNTIME MAIN WINDOW TRIGGER
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let startup = app::snapshot::Startup::from_args();
    let snapshot = startup.snapshot.as_ref().map(|r| r.path.clone());
    let size = match startup.snapshot {
        Some(_) => app::snapshot::WINDOW,
        None => [1150.0, 720.0],
    };
    // let _ = env_logger::Builder::from_env(
    //     env_logger::Env::default().default_filter_or("info,octant=debug,wgpu=warn,zarrs=warn"),
    // )
    // .try_init();

    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(size)
        .with_title(app::window_title::title_in(i18n::language()));
    // macOS keeps the app bundle's .icns in the Dock: eframe leaves the Dock icon alone for an
    // empty icon, and would otherwise put its own egui icon (no icon) or this edge-to-edge one there.
    #[cfg(target_os = "macos")]
    let viewport = viewport.with_icon(egui::IconData::default());
    // The taskbar icon on Windows and Linux.
    #[cfg(not(target_os = "macos"))]
    let viewport = match eframe::icon_data::from_png_bytes(include_bytes!(
        "../assets/brand/png/octant-app-icon-512.png"
    )) {
        Ok(icon) => viewport.with_icon(icon),
        Err(_) => viewport,
    };

    let native_options = eframe::NativeOptions {
        viewport,
        depth_buffer: 32,
        wgpu_options: plots::device::wgpu_configuration(),
        // Persistence stores colormap preferences only; keep window geometry unmanaged.
        persist_window: false,
        // `ui::fonts` installs its own system font provider, asking for the
        // locale of the Han font it picks rather than the system's.
        system_font_fallback: false,
        ..Default::default()
    };

    let started = std::time::SystemTime::now();
    eframe::run_native(
        "Octant",
        native_options,
        Box::new(|cc| {
            let mut app = OctantApp::new(cc);
            app.start(startup);
            Ok(Box::new(app))
        }),
    )?;
    app::snapshot::exit_unless_saved(snapshot.as_deref(), started);
    Ok(())
}

// BROWSER INTERFACE MOUNT BRIDGE
#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    let web_options = eframe::WebOptions {
        depth_buffer: 32,
        wgpu_options: plots::device::wgpu_configuration(),
        ..Default::default()
    };
    wasm_bindgen_futures::spawn_local(async {
        use wasm_bindgen::JsCast;
        let Some(window) = web_sys::window() else {
            log::error!("Unable to start Octant: browser window is unavailable");
            return;
        };
        let Some(document) = window.document() else {
            log::error!("Unable to start Octant: document is unavailable");
            return;
        };
        let Some(canvas_element) = document.get_element_by_id("octant_canvas_anchor") else {
            log::error!("Unable to start Octant: canvas element is missing");
            return;
        };
        let Ok(canvas) = canvas_element.dyn_into::<web_sys::HtmlCanvasElement>() else {
            log::error!("Unable to start Octant: target element is not a canvas");
            return;
        };

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
