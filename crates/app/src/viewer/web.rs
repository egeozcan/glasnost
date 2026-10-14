//! Browser side of the viewer: the folder picker.
//!
//! The page holds a hidden `<input type="file" webkitdirectory>` with id
//! [`PICKER_ID`]. The viewer's button clicks it; when the user picks a
//! folder, its files become [`PickedFile`]s read lazily with `File.slice`.

use std::rc::Rc;

use glasnost_vfs::PickedFile;
use glasnost_vfs::source::WebFileSource;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use winit::event_loop::EventLoopProxy;

use super::ViewerEvent;
use super::tasks;

/// Id of the folder input element in `web/index.html`.
pub const PICKER_ID: &str = "glasnost-folder";

fn picker() -> Option<web_sys::HtmlInputElement> {
    web_sys::window()?
        .document()?
        .get_element_by_id(PICKER_ID)?
        .dyn_into()
        .ok()
}

/// Opens the browser's folder dialog.
pub fn open_folder_picker() {
    if let Some(input) = picker() {
        input.click();
    }
}

/// Mounts whatever folder the user picks.
pub(super) fn install_folder_picker(proxy: EventLoopProxy<ViewerEvent>) {
    on_folder_picked(move |files| {
        let proxy = proxy.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = tasks::mount(files).await;
            let _ = proxy.send_event(ViewerEvent::Task(result));
        });
    });
}

/// Calls `f` with the files of each folder the user picks.
pub(crate) fn on_folder_picked(mut f: impl FnMut(Vec<PickedFile>) + 'static) {
    let Some(input) = picker() else {
        log::warn!("the page has no #{PICKER_ID} input; folder picking is unavailable");
        return;
    };
    let target = input.clone();
    let on_change = Closure::<dyn FnMut()>::new(move || {
        let Some(list) = target.files() else { return };
        let mut files = Vec::new();
        for i in 0..list.length() {
            let Some(file) = list.get(i) else { continue };
            // `webkitRelativePath` is "Folder/sub/name"; drop the picked
            // folder's own name so top-level files have no '/'.
            let rel = js_sys::Reflect::get(&file, &"webkitRelativePath".into())
                .ok()
                .and_then(|v| v.as_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| file.name());
            let path = rel
                .split_once('/')
                .map_or(rel.as_str(), |(_, p)| p)
                .to_owned();
            files.push(PickedFile {
                path,
                source: Rc::new(WebFileSource::new(file)),
            });
        }
        f(files);
    });
    input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
    on_change.forget();
}
