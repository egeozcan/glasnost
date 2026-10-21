//! The browser's way to store a desync report (M9 N4): the report's bytes
//! become a download through a Blob URL and a clicked anchor; the browser
//! puts it in its downloads. Our own netcode.

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

/// How long the Blob URL lives after the click, for the download to start.
// nocite: our own netcode
const REVOKE_AFTER_MS: i32 = 10_000;

/// Offers `bytes` as a download named `name`; returns where it went.
pub fn save(name: &str, bytes: &[u8]) -> Result<String, String> {
    let js = |e: JsValue| format!("{e:?}");
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::from(bytes));
    let opts = BlobPropertyBag::new();
    opts.set_type("application/octet-stream");
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts).map_err(js)?;
    let url = Url::create_object_url_with_blob(&blob).map_err(js)?;
    let a: HtmlAnchorElement = document
        .create_element("a")
        .map_err(js)?
        .dyn_into()
        .map_err(|_| "not an anchor".to_owned())?;
    a.set_href(&url);
    a.set_download(name);
    a.click();
    let revoke = Closure::once_into_js(move || {
        let _ = Url::revoke_object_url(&url);
    });
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            revoke.unchecked_ref(),
            REVOKE_AFTER_MS,
        )
        .map_err(js)?;
    Ok(format!("your downloads as {name}"))
}
