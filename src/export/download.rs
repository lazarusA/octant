//! Browser downloads of exported figures (the web has no filesystem).

use wasm_bindgen::JsCast;

/// Hands `data` to the browser as a download named `filename`.
pub fn download(data: &[u8], filename: &str, mime: &str) -> Result<(), String> {
    let js_err = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(data));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime);
    let blob =
        web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).map_err(js_err)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(js_err)?;

    let anchor = document
        .create_element("a")
        .map_err(js_err)?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "not an anchor element")?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    // Revoked later: some browsers (Safari) cancel a download whose URL is
    // revoked while the click is still being handled.
    let revoke = wasm_bindgen::closure::Closure::once_into_js(move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    });
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(revoke.unchecked_ref(), REVOKE_MS)
        .map_err(js_err)?;
    Ok(())
}

/// How long the download's object URL stays valid after the click.
const REVOKE_MS: i32 = 60_000;
