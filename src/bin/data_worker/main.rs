//! The data worker: loads blocks in a browser Web Worker, so fetching and
//! decoding never hold the page's frames (`octant::data::worker`). Trunk
//! builds it as its own `.wasm`, fetched when a dataset starts opening.
//! Nothing on the desktop.

#[cfg(not(target_arch = "wasm32"))]
fn main() {}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);
    octant::data::worker::host::start();
}
