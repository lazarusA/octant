//! Picking a local folder or file. A folder comes from `showDirectoryPicker`
//! where the browser has it (Chromium: nothing is listed, keys are looked up
//! when read), else from a `webkitdirectory` input (Firefox, Safari: the
//! browser lists every file before handing them over).
//!
//! Both need a recent user gesture; egui runs a click's handler in the next
//! animation frame, well within the seconds browsers allow after it.

use super::registry::{Source, open};
use js_sys::{Function, Reflect};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};

/// Asks for a folder; once picked it opens as a dataset (`take_opened`).
pub fn pick_folder(ctx: &egui::Context) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let picker = Reflect::get(&window, &"showDirectoryPicker".into()).ok();
    let Some(picker) = picker.and_then(|p| p.dyn_into::<Function>().ok()) else {
        pick_with_input(ctx, true);
        return;
    };
    let ctx = ctx.clone();
    spawn_local(async move {
        let Ok(promise) = picker.call0(&window) else {
            return;
        };
        // A cancelled picker rejects (`AbortError`): nothing to open.
        let Ok(handle) = JsFuture::from(js_sys::Promise::from(promise)).await else {
            return;
        };
        let name = Reflect::get(&handle, &"name".into())
            .ok()
            .and_then(|n| n.as_string())
            .unwrap_or_default();
        open(name, Source::Handle(handle), ctx).await;
    });
}

/// Asks for a GeoTIFF, NetCDF, HDF5 or MATLAB file; once picked it opens as a dataset.
pub fn pick_file(ctx: &egui::Context) {
    pick_with_input(ctx, false);
}

/// A hidden file input, `folder` taking a whole folder; removed once it
/// hands over its files (a cancelled one stays hidden in the page).
fn pick_with_input(ctx: &egui::Context, folder: bool) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(input) = document.create_element("input") else {
        return;
    };
    let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>() else {
        return;
    };
    input.set_type("file");
    if folder {
        let _ = input.set_attribute("webkitdirectory", "");
    } else {
        input.set_accept(
            ".tif,.tiff,.geotif,.geotiff,.cog,.nc,.nc4,.cdf,.netcdf,.h5,.hdf5,.hdf,.he5,.mat",
        );
    }
    let _ = input.set_attribute("style", "display: none");
    if let Some(body) = document.body() {
        let _ = body.append_child(&input);
    }
    let ctx = ctx.clone();
    let target = input.clone();
    let on_change = Closure::once_into_js(move || {
        target.remove();
        let Some(files) = target.files() else {
            return;
        };
        if let Some((name, source)) = picked(&files, folder) {
            spawn_local(open(name, source, ctx));
        }
    });
    input.set_onchange(Some(on_change.unchecked_ref()));
    input.click();
}

/// The name and source of the files an input handed over: a folder's files
/// by their path under it, or the one file.
fn picked(files: &web_sys::FileList, folder: bool) -> Option<(String, Source)> {
    if !folder {
        let file = files.get(0)?;
        return Some((file.name(), Source::File(file)));
    }
    let mut name = String::new();
    let mut by_path = HashMap::new();
    for i in 0..files.length() {
        let Some(file) = files.get(i) else { continue };
        let relative = relative_path(&file);
        let Some((root, key)) = relative.split_once('/') else {
            continue;
        };
        if name.is_empty() {
            name = root.to_owned();
        }
        by_path.insert(key.to_owned(), file);
    }
    (!by_path.is_empty()).then(|| (name, Source::Listed(Rc::new(by_path))))
}

/// A listed file's path from the picked folder on (`root/a/zarr.json`).
fn relative_path(file: &web_sys::File) -> String {
    Reflect::get(file, &JsValue::from_str("webkitRelativePath"))
        .ok()
        .and_then(|p| p.as_string())
        .unwrap_or_default()
}
