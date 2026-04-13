use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MediaRecorder, MediaStream, BlobEvent, MediaRecorderOptions};
use std::rc::Rc;
use std::cell::RefCell;
use leptos::Callback;

pub struct LocalRecorder {
    recorder: MediaRecorder,
    _on_data_available: Closure<dyn FnMut(BlobEvent)>,
    _on_stop: Closure<dyn FnMut()>,
}

impl LocalRecorder {
    pub fn new(stream: MediaStream, _on_error: Callback<String>) -> Result<Self, JsValue> {
        let options = MediaRecorderOptions::new();
        options.set_mime_type("video/webm;codecs=vp8,opus");

        let recorder = MediaRecorder::new_with_media_stream_and_media_recorder_options(&stream, &options)?;

        let chunks: Rc<RefCell<Vec<web_sys::Blob>>> = Rc::new(RefCell::new(Vec::new()));

        let chunks_clone = chunks.clone();
        let on_data_available = Closure::wrap(Box::new(move |e: BlobEvent| {
            if let Some(blob) = e.data() {
                if blob.size() > 0.0 {
                    chunks_clone.borrow_mut().push(blob);
                }
            }
        }) as Box<dyn FnMut(BlobEvent)>);

        let chunks_clone_2 = chunks.clone();
        let on_stop = Closure::wrap(Box::new(move || {
            let blob_parts = js_sys::Array::new();
            for blob in chunks_clone_2.borrow().iter() {
                blob_parts.push(blob);
            }

            let property_bag = web_sys::BlobPropertyBag::new();
            property_bag.set_type("video/webm");
            if let Ok(blob) = web_sys::Blob::new_with_blob_sequence_and_options(
                &blob_parts,
                &property_bag
            ) {
                if let Some(window) = web_sys::window() {
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                        let document = window.document().unwrap();
                        let a = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                        a.set_href(&url);
                        a.set_download(&format!("juncto-recording-{}.webm", js_sys::Date::now()));
                        a.click();
                        let _ = web_sys::Url::revoke_object_url(&url);
                    }
                }
            }
            chunks_clone_2.borrow_mut().clear();
        }) as Box<dyn FnMut()>);

        recorder.set_ondataavailable(Some(on_data_available.as_ref().unchecked_ref()));
        recorder.set_onstop(Some(on_stop.as_ref().unchecked_ref()));

        recorder.start_with_time_slice(5000)?; // 5s slices

        Ok(Self {
            recorder,
            _on_data_available: on_data_available,
            _on_stop: on_stop,
        })
    }

    pub fn stop(&self) {
        let _ = self.recorder.stop();
    }
}

impl Drop for LocalRecorder {
    fn drop(&mut self) {
        let _ = self.recorder.stop();
    }
}
