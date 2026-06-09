use leptos::{Callable, Callback};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{BlobEvent, MediaRecorder, MediaRecorderOptions, MediaStream};

pub struct LocalRecorder {
    recorder: MediaRecorder,
    _on_data_available: Closure<dyn FnMut(BlobEvent)>,
    _on_stop: Closure<dyn FnMut()>,
    _on_error: Closure<dyn FnMut(web_sys::Event)>,
}

impl LocalRecorder {
    pub fn new(stream: MediaStream, on_error: Callback<String>) -> Result<Self, JsValue> {
        let options = MediaRecorderOptions::new();
        options.set_mime_type("video/webm;codecs=vp8,opus");

        let recorder =
            MediaRecorder::new_with_media_stream_and_media_recorder_options(&stream, &options)?;

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
            if let Ok(blob) =
                web_sys::Blob::new_with_blob_sequence_and_options(&blob_parts, &property_bag)
            {
                if let Some(window) = web_sys::window() {
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                        let document = window.document().unwrap();
                        let a = document
                            .create_element("a")
                            .unwrap()
                            .dyn_into::<web_sys::HtmlAnchorElement>()
                            .unwrap();
                        a.set_href(&url);
                        a.set_download(&format!("juncto-recording-{}.webm", js_sys::Date::now()));
                        a.click();
                        // Delay revoking the object URL so the browser has time
                        // to fully initiate the download. Revoking synchronously
                        // after click() can cause download failures in some
                        // browser environments.
                        let url_to_revoke = url;
                        let revoke_cb = Closure::once(move || {
                            let _ = web_sys::Url::revoke_object_url(&url_to_revoke);
                        });
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            revoke_cb.as_ref().unchecked_ref(),
                            1000,
                        );
                        revoke_cb.forget();
                    }
                }
            }
            chunks_clone_2.borrow_mut().clear();
        }) as Box<dyn FnMut()>);

        let on_error_cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let msg = js_sys::Reflect::get(&e, &JsValue::from_str("message"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "Unknown MediaRecorder error".to_string());
            on_error.call(msg);
        }) as Box<dyn FnMut(web_sys::Event)>);

        recorder.set_ondataavailable(Some(on_data_available.as_ref().unchecked_ref()));
        recorder.set_onstop(Some(on_stop.as_ref().unchecked_ref()));
        let error_target: &web_sys::EventTarget = recorder.unchecked_ref();
        let _ = error_target
            .add_event_listener_with_callback("error", on_error_cb.as_ref().unchecked_ref());

        recorder.start_with_time_slice(5000)?; // 5s slices

        Ok(Self {
            recorder,
            _on_data_available: on_data_available,
            _on_stop: on_stop,
            _on_error: on_error_cb,
        })
    }

    pub fn stop(&self) {
        let _ = self.recorder.stop();
    }
}

impl Drop for LocalRecorder {
    fn drop(&mut self) {
        // Only call stop() if the recorder is still active. Calling stop() on
        // an already-stopped MediaRecorder throws a DOMException.
        if self.recorder.state() == web_sys::RecordingState::Recording
            || self.recorder.state() == web_sys::RecordingState::Paused
        {
            let _ = self.recorder.stop();
        }
        // Remove the error event listener added via addEventListener so
        // the browser doesn't try to invoke the dropped Closure.
        let error_target: &web_sys::EventTarget = self.recorder.unchecked_ref();
        let _ = error_target
            .remove_event_listener_with_callback("error", self._on_error.as_ref().unchecked_ref());
        // Clear property-based handlers for the same reason.
        self.recorder.set_ondataavailable(None);
        self.recorder.set_onstop(None);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_media_recorder_compiles() {
        let _ = true;
    }
}
