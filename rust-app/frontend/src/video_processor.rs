use wasm_bindgen::prelude::*;
use web_sys::{MediaStream};

#[wasm_bindgen(module = "/public/video_processor.js")]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object)]
    pub type VideoProcessor;

    #[wasm_bindgen(js_name = createVideoProcessor)]
    pub fn create_video_processor() -> VideoProcessor;

    #[wasm_bindgen(method, catch)]
    pub async fn start(this: &VideoProcessor, stream: &MediaStream) -> Result<(), JsValue>;

    #[wasm_bindgen(method)]
    pub fn stop(this: &VideoProcessor);

    #[wasm_bindgen(method, js_name = setMode)]
    pub fn set_mode(this: &VideoProcessor, mode: &str, image_url: Option<String>);

    #[wasm_bindgen(method, js_name = getStream)]
    pub fn get_stream(this: &VideoProcessor) -> MediaStream;
}
